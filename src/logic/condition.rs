use std::{collections::{HashMap, HashSet}, io::Read};
use serde::Deserialize;
use crate::logic_parse::{parse_reqs, gen_reqs2};

use super::{ItemId, FlagId, Flag, LocationId, Location, ItemCategory, ItemDef, Restriction, Logic};

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
    Item(ItemId, usize),
    AtLeast(usize, Vec<(ItemId, usize)>),
    And(Vec<Self>),
    Or(Vec<Self>),
}


impl Condition {
    pub fn expand(self, cond_fact: &Logic) -> ItemCondition {
        match self {
            Condition::NoRequirements => ItemCondition::NoRequirements,
            Condition::Flag(id) => {
                let flag = cond_fact.get_flag(id).unwrap();
                flag.requirement.clone()
            },
            Condition::AtLeast(count, items) => ItemCondition::AtLeast(count, items),
            Condition::Item(id, count) => ItemCondition::Item(id, count),
            Condition::Location(id) => {
                let location = cond_fact.get_location(id).unwrap();
                location.requirement.clone()
            }
            Condition::And(conds) => {
                ItemCondition::And(conds.into_iter().map(|c| c.expand(cond_fact)).collect())
            },
            Condition::Or(conds) => ItemCondition::Or(conds.into_iter().map(|c| c.expand(cond_fact)).collect()),
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
    fn top_level_items(&self) -> HashSet<ItemId> {
        match self {
            Self::NoRequirements => HashSet::new(),
            Self::Item(id, _) => {
                let mut rv = HashSet::with_capacity(1);
                rv.insert(*id);
                rv
            },
            Self::AtLeast(_, items) => {
                let mut rv = HashSet::with_capacity(1);
                for (id, _) in items {
                    rv.insert(*id);
                }
                rv
            }
            Self::And(conds) => conds.iter().filter_map(|c| match c { Self::Item(id, _) => Some(*id), _ => None }).collect(),
            Self::Or(conds) => conds.iter().filter_map(|c| match c { Self::Item(id, _) => Some(*id), _ => None }).collect(),
        }
    }
    pub fn missing(&self, items: &HashMap<ItemId, usize>, rv: &mut HashMap<ItemId, usize>) {
        if !self.satisfied(items) {
            match self {
                ItemCondition::NoRequirements => {},
                ItemCondition::Item(id, count) => {rv.insert(*id, *count);}
                ItemCondition::AtLeast(threshold, req_items) => {
                    let mut total = 0;
                        
                    for &(id, weight) in req_items {
                        total += items.get(&id).copied().unwrap_or(0) * weight;
                    }
                    if *threshold <= total {
                        // This should never happen, since we're unsatisfied
                        return;
                    }
                    let required_threshold = threshold - total;
                    
                    for &(id, weight) in req_items {
                        rv.insert(id, (required_threshold + weight - 1) / weight);
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
                let items: Vec<_> = new_conds.iter().filter_map(|c| match c { Self::Item(id, count) => Some((*id, *count)), _ => None }).collect();
                for (item, count) in items {
                    for cond in &mut new_conds {
                        if matches!(cond, Self::Or(_) | Self::And(_)) {
                            cond.assume_item(item, count);
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
    pub fn assume_item(&mut self, id: ItemId, count: usize) {
        match self {
            Self::NoRequirements => {}
            Self::Item(my_id, my_count) => if id == *my_id {
                if count >= *my_count {
                    *self = Self::NoRequirements;
                } else {
                    *my_count -= count;
                }
            },
            Self::AtLeast(threshold, items) => {
                if let Some((_, weight)) = items.iter().find(|(i, _)| *i == id) {
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

    pub fn render(&self, cond_fact: &Logic) {
        match self {
            Self::NoRequirements => print!("-"),
            Self::Item(id, count) => {
                let item = cond_fact.get_item(*id).unwrap();
                if *count > 1 {
                    print!("{}:{}", item.name, count);
                } else {
                    print!("{}", item.name);
                }
            }
            Self::AtLeast(threshold, items) => {
                let mut first = true;
                print!("({} <=", threshold);
                for (id, weight) in items {
                    let item = cond_fact.get_item(*id).unwrap();
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
                    cond.render(cond_fact);
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
                    cond.render(cond_fact);
                }
                print!(")");
            },
        }
    }
    pub fn satisfied(&self, items: &HashMap<ItemId, usize>) -> bool {
        match self {
            Self::NoRequirements => true,
            Self::AtLeast(threshold, req_items) => {
                let mut total = 0;
                for &(id, weight) in req_items {
                    total += items.get(&id).copied().unwrap_or(0) * weight;
                }

                total >= *threshold
            }
            Self::Item(id, count) => items.get(id).copied().unwrap_or(0) >= *count,
            Self::Or(conds) => conds.iter().any(|cond| cond.satisfied(items)),
            Self::And(conds) => conds.iter().all(|cond| cond.satisfied(items)),
        }
    }

    pub fn satisfied_by(&self, cond_fact: &Logic, items: &HashMap<ItemId, usize>) -> HashSet<ItemId> {
        match self {
            Self::NoRequirements => HashSet::new(),
            Self::Item(id, count) => {
                let mut rv = HashSet::new();
                if items.get(id).copied().unwrap_or(0) >= *count {
                    rv.insert(*id);
                }
                rv
            }
            Self::AtLeast(threshold, req_items) => {
                let mut total = 0;
                for &(id, weight) in req_items {
                    total += items.get(&id).copied().unwrap_or(0) * weight;
                }
                let mut rv = HashSet::new();
                if total >= *threshold {
                    for &(id, _weight) in req_items {
                        if items.get(&id).copied().unwrap_or(0) > 0 {
                            rv.insert(id);
                        }
                    }
                }
                rv
            }
            Self::And(conds) => {
                let mut rv = HashSet::new();
                for cond in conds {
                    rv.extend(cond.satisfied_by(cond_fact, items));
                }
                rv
            },
            Self::Or(conds) => {
                for cond in conds {
                    if cond.satisfied(items) {
                        return cond.satisfied_by(cond_fact, items);
                    }
                }
                HashSet::new()
            }
        }
    }
}
