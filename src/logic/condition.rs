use std::{collections::{HashMap, HashSet}, rc::Rc};
use serde::Deserialize;
use crate::logic_parse::{parse_reqs, gen_reqs2};

use super::{ItemId, FlagId, PreFlag, LocationId, PreLocation, ItemCategory, ItemDef, Restriction, PreLogic};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Condition {
    NoRequirements,
    Flag(FlagId),
    Item(ItemId, usize),
    AtLeast(usize, Vec<(ItemId, usize)>),
    Location(LocationId),
    And(Vec<Self>),
    Or(Vec<Self>),
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ItemCondition {
    NoRequirements,
    Item(Rc<ItemDef>, usize),
    AtLeast(usize, Vec<(Rc<ItemDef>, usize)>),
    And(Vec<Self>),
    Or(Vec<Self>),
}


impl Condition {
    pub fn expand<'a>(&'a self, prelogic: &'a PreLogic, condition_cache: &mut HashMap<&'a Condition, ItemCondition>, item_cache: &HashMap<ItemId, Rc<ItemDef>>) -> ItemCondition {
        if let Some(condition) = condition_cache.get(self) {
            condition.clone()
        } else {
            let condition = match self {
                Condition::NoRequirements => ItemCondition::NoRequirements,
                Condition::Flag(id) => {
                    let flag = prelogic.get_flag(*id).unwrap();
                    flag.requirement.expand(prelogic, condition_cache, item_cache)
                },
                Condition::AtLeast(count, items) => ItemCondition::AtLeast(
                    *count,
                    items.iter().map(|(id, weight)| (item_cache[id].clone(), *weight)).collect()
                ),
                Condition::Item(id, count) => ItemCondition::Item(item_cache[id].clone(), count.clone()),
                Condition::Location(id) => {
                    let location = prelogic.get_location(*id).unwrap();
                    location.requirement.expand(prelogic, condition_cache, item_cache)
                }
                Condition::And(conds) => {
                    ItemCondition::And(conds.into_iter().map(|c| c.expand(prelogic, condition_cache, item_cache)).collect())
                },
                Condition::Or(conds) => ItemCondition::Or(conds.into_iter().map(|c| c.expand(prelogic, condition_cache, item_cache)).collect()),
            };
            condition_cache.insert(self, condition.clone());
            condition
        }
    }
}
impl ItemCondition {
    pub fn complexity(&self) -> u32 {
        match self {
            Self::NoRequirements => 0,
            Self::Item(_, _) => 1,
            Self::AtLeast(_, items) => items.len() as _,
            Self::And(conds) => conds.iter().map(|c| c.complexity()).sum::<u32>() + 1,
            Self::Or(conds) => conds.iter().map(|c| c.complexity()).min().unwrap_or(0) + 1
        }
    }
    pub fn missing(&self, items: &HashMap<Rc<ItemDef>, usize>, rv: &mut HashMap<Rc<ItemDef>, usize>) {
        if !self.satisfied(items) {
            match self {
                ItemCondition::NoRequirements => {},
                ItemCondition::Item(id, count) => {rv.insert(id.clone(), *count);}
                ItemCondition::AtLeast(threshold, req_items) => {
                    let mut total = 0;
                        
                    for (item, weight) in req_items {
                        total += items.get(item).copied().unwrap_or(0) * *weight;
                    }
                    if *threshold <= total {
                        // This should never happen, since we're unsatisfied
                        return;
                    }
                    let required_threshold = threshold - total;
                    
                    for (id, weight) in req_items {
                        rv.insert(id.clone(), (required_threshold + weight - 1) / weight);
                    }                    
                }
                ItemCondition::And(conds) => {
                    for cond in conds {
                        cond.missing(items, rv);
                    }
                },
                ItemCondition::Or(conds) => {
                    for cond in conds {
                        cond.missing(items, rv);
                    }
                },
            }
        }
        
    }
    pub fn flatten(&self) -> Self {
        match self {
            Self::NoRequirements => self.clone(),
            Self::Item(_, _) => self.clone(),
            Self::AtLeast(_, _) => self.clone(),
            Self::And(conds) => {
                let mut new_conds = vec![];
                for cond in conds {
                    let new_cond = cond.flatten();
                    match new_cond {
                        Self::And(conds) => new_conds.extend(conds),
                        _ => new_conds.push(new_cond)
                    }
                }
                new_conds.sort();
                new_conds.dedup();
                Self::And(new_conds)
            },
            Self::Or(conds) => {
                let mut new_conds = vec![];
                for cond in conds {
                    let new_cond = cond.flatten();
                    match new_cond {
                        Self::Or(conds) => new_conds.extend(conds),
                        _ => new_conds.push(new_cond)
                    }
                }
                new_conds.sort();
                new_conds.dedup();
                Self::Or(new_conds)
            }
        }
    }
    fn remove_redundant_ors(&mut self, or: &[Self]) -> bool {
        let mut changed = false;
        match self {
            Self::And(conds) => {
                for cond in &mut *conds {
                    changed |= cond.remove_redundant_ors(or);
                }
                if changed {
                    conds.retain(|c| *c != Self::NoRequirements);
                    match conds.len() {
                        0 => *self = Self::NoRequirements,
                        1 => *self = conds.pop().expect("Looked before we lept"),
                        _ => {}
                    }
                }
            },
            Self::Or(conds) => {
                if or.iter().all(|c| conds.contains(c)) {
                    *self = Self::NoRequirements;
                    changed = true;
                } else {
                    for cond in &mut *conds {
                        changed |= cond.remove_redundant_ors(or);
                    }
                    if changed {
                        conds.retain(|c| *c != Self::NoRequirements);
                        match conds.len() {
                            0 => *self = Self::NoRequirements,
                            1 => *self = conds.pop().expect("Looked before we lept"),
                            _ => {}
                        }
                    }
                }
            }
            _ => {}
        }
        changed
    }
    pub fn simplify(&self) -> Self {
        match self {
            Self::NoRequirements => self.clone(),
            Self::Item(_, _) => self.clone(),
            Self::AtLeast(_, _) => self.clone(),
            Self::And(conds) => {
                let mut new_conds = vec![];
                for cond in conds {
                    let new_cond = cond.flatten();
                    match new_cond {
                        Self::And(conds) => new_conds.extend(conds),
                        _ => new_conds.push(new_cond)
                    }
                }
                let items: Vec<_> = new_conds.iter().filter_map(|c| match c { Self::Item(id, count) => Some((id.clone(), *count)), _ => None }).collect();
                for (item, count) in items {
                    for cond in &mut new_conds {
                        if matches!(cond, Self::Or(_) | Self::And(_)) {
                            cond.assume_item(&item, count);
                        }
                    }
                }
                new_conds.retain(|c| *c != Self::NoRequirements);
                new_conds.sort();
                new_conds.dedup();
                let disj: Vec<_> = new_conds.iter().filter_map(|c| match c { Self::Or(conds) => Some(conds.clone()), _ => None }).collect();
                for cond in &mut new_conds {
                    if matches!(cond, Self::And(_) | Self::Or(_)) {
                        for or in &disj {
                            if matches!(cond, Self::Or(cond) if cond == or) {
                                continue;
                            }
                            cond.remove_redundant_ors(or);
                        }
                    }                    
                }
                new_conds.retain(|c| *c != Self::NoRequirements);
                new_conds.sort();
                new_conds.dedup();
                
                let rv = Self::And(new_conds);
                rv
            },
            Self::Or(conds) => {
                let mut conds = conds.clone();
                conds.sort();
                conds.dedup();
                Self::Or(conds)
            }
        }
    }
    pub fn assume_item(&mut self, id: &Rc<ItemDef>, count: usize) {
        match self {
            Self::NoRequirements => {}
            Self::Item(my_id, my_count) => if id == my_id {
                if count >= *my_count {
                    *self = Self::NoRequirements;
                } else {
                    *my_count -= count;
                }
            },
            Self::AtLeast(threshold, items) => {
                if let Some((_, weight)) = items.iter().find(|(i, _)| i == id) {
                    if *threshold <= count * *weight {
                        *self = Self::NoRequirements;
                    } else {
                        *threshold -= count * *weight;
                    }
                }
            }
            Self::And(conds) => {
                for cond in &mut *conds {
                    cond.assume_item(id, count);
                }
                conds.retain(|c| *c != Self::NoRequirements);
                if conds.len() == 0 {
                    *self = Self::NoRequirements;
                } else if conds.len() == 1 {
                    *self = conds.pop().expect("We just checked that conds is non-empty");
                }
            }
            Self::Or(conds) => {
                for cond in &mut *conds {
                    cond.assume_item(id, count);
                }
                if conds.iter().any(|c| *c == Self::NoRequirements) {
                    *self = Self::NoRequirements;
                }
            }
        }
    }

    pub fn render(&self) {
        match self {
            Self::NoRequirements => print!("-"),
            Self::Item(item, count) => {
                if *count > 1 {
                    print!("{}:{}", item.name, count);
                } else {
                    print!("{}", item.name);
                }
            }
            Self::AtLeast(threshold, items) => {
                let mut first = true;
                print!("({} <=", threshold);
                for (item, weight) in items {
                    if !first {
                        print!(" + ")
                    }
                    first = false;
                    print!("{}", item.name);
                    if *weight > 1 {
                        print!(" * {}", weight)
                    }
                }
                print!(")");
            }
            Self::And(conds) => {
                let mut first = true;
                print!("(");
                for cond in conds {
                    if !first {
                        print!(" & ")
                    }
                    first = false;
                    cond.render();
                }
                print!(")");
            }
            Self::Or(conds) => {
                let mut first = true;
                print!("(");
                for cond in conds {
                    if !first {
                        print!(" | ")
                    }
                    first = false;
                    cond.render();
                }
                print!(")");
            },
        }
    }
    pub fn satisfied(&self, items: &HashMap<Rc<ItemDef>, usize>) -> bool {
        match self {
            Self::NoRequirements => true,
            Self::AtLeast(threshold, req_items) => {
                let mut total = 0;
                for (id, weight) in req_items {
                    total += items.get(id).copied().unwrap_or(0) * *weight;
                }

                total >= *threshold
            }
            Self::Item(id, count) => items.get(id).copied().unwrap_or(0) >= *count,
            Self::Or(conds) => conds.iter().any(|cond| cond.satisfied(items)),
            Self::And(conds) => conds.iter().all(|cond| cond.satisfied(items)),
        }
    }

    pub fn satisfied_by(&self, items: &HashMap<Rc<ItemDef>, usize>) -> HashSet<Rc<ItemDef>> {
        match self {
            Self::NoRequirements => HashSet::new(),
            Self::Item(item, count) => {
                let mut rv = HashSet::new();
                if items.get(item).copied().unwrap_or(0) >= *count {
                    rv.insert(item.clone());
                }
                rv
            }
            Self::AtLeast(threshold, req_items) => {
                let mut total = 0;
                for (item, weight) in req_items {
                    total += items.get(item).copied().unwrap_or(0) * *weight;
                }
                let mut rv = HashSet::new();
                if total >= *threshold {
                    for (id, _weight) in req_items {
                        if items.get(id).copied().unwrap_or(0) > 0 {
                            rv.insert(id.clone());
                        }
                    }
                }
                rv
            }
            Self::And(conds) => {
                let mut rv = HashSet::new();
                for cond in conds {
                    rv.extend(cond.satisfied_by(items));
                }
                rv
            },
            Self::Or(conds) => {
                for cond in conds {
                    if cond.satisfied(items) {
                        return cond.satisfied_by(items);
                    }
                }
                HashSet::new()
            }
        }
    }
}
