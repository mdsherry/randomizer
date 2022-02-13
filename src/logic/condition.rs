use std::{collections::{HashMap, HashSet}, rc::Rc, vec};
use joinery::JoinableIterator;

use super::{ItemId, FlagId, LocationId, ItemDef, PreLogic};

pub type Conditional<T> = (Rc<T>, ItemCondition);
pub type Conditionals<T> = Vec<Conditional<T>>;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Condition {
    NoRequirements,
    Flag(FlagId),
    Item(ItemId, usize),
    Parameter(String),
    AtLeast(usize, Vec<(ItemId, usize)>),
    Location(LocationId),
    And(Vec<Self>),
    Or(Vec<Self>),
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ItemCondition {
    NoRequirements,
    Unattainable,
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
                Condition::Parameter(name) => if prelogic.get_parameter(name) { ItemCondition::NoRequirements } else { ItemCondition::Unattainable },
                Condition::Item(id, count) => ItemCondition::Item(item_cache[id].clone(), *count),
                Condition::Location(id) => {
                    let location = prelogic.get_location(*id).unwrap();
                    location.requirement.expand(prelogic, condition_cache, item_cache)
                }
                Condition::And(conds) => {
                    ItemCondition::And(conds.iter().map(|c| c.expand(prelogic, condition_cache, item_cache)).collect())
                },
                Condition::Or(conds) => ItemCondition::Or(conds.iter().map(|c| c.expand(prelogic, condition_cache, item_cache)).collect()),
            };
            let condition = condition.simplify().flatten();
            condition_cache.insert(self, condition.clone());
            condition
        }
    }
}
impl ItemCondition {
    pub fn complexity(&self) -> u32 {
        match self {
            Self::NoRequirements | Self::Unattainable => 0,
            Self::Item(_, _) => 1,
            Self::AtLeast(_, items) => items.len() as _,
            Self::And(conds) => conds.iter().map(|c| c.complexity()).sum::<u32>() + 1,
            Self::Or(conds) => conds.iter().map(|c| c.complexity()).min().unwrap_or(0) + 1
        }
    }
    pub fn missing(&self, rv: &mut HashMap<Rc<ItemDef>, usize>) {
        // if !self.satisfied(items) {
            match self {
                ItemCondition::NoRequirements | Self::Unattainable => {},
                ItemCondition::Item(id, count) => {rv.insert(id.clone(), *count);}
                ItemCondition::AtLeast(threshold, req_items) => {
                    for (id, weight) in req_items {
                        rv.insert(id.clone(), (threshold + weight - 1) / weight);
                    }                    
                }
                ItemCondition::And(conds) => {
                    for cond in conds {
                        cond.missing(rv);
                    }
                },
                ItemCondition::Or(conds) => {
                    for cond in conds {
                        cond.missing(rv);
                    }
                },
            }
        // }
        
    }
    pub fn flatten(&self) -> Self {
        match self {
            Self::NoRequirements | Self::Unattainable  => self.clone(),
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
                match new_conds.len() {
                    0 => Self::NoRequirements,
                    1 => new_conds.pop().unwrap(),
                    _ => Self::And(new_conds)
                }
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
                match new_conds.len() {
                    0 => Self::Unattainable,
                    1 => new_conds.pop().unwrap(),
                    _ => Self::Or(new_conds)
                }
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
            Self::NoRequirements | Self::Unattainable  => self.clone(),
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
                if new_conds.contains(&ItemCondition::Unattainable) {
                    ItemCondition::Unattainable
                } else {
                    Self::And(new_conds).flatten()
                }
            },
            Self::Or(conds) => {
                let mut conds = conds.clone();
                conds.sort();
                conds.dedup();
                conds.retain(|c| *c != ItemCondition::Unattainable);
                if conds.contains(&ItemCondition::NoRequirements) {
                    ItemCondition::NoRequirements
                } else {
                    Self::Or(conds).flatten()
                }                
            }
        }
    }
    pub fn assume_item(&mut self, id: &Rc<ItemDef>, count: usize) {
        match self {
            Self::NoRequirements | Self::Unattainable  => {}
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
                if conds.is_empty() {
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
    pub fn min_sat<'a>(&'a self, items: &HashMap<&'a Rc<ItemDef>, usize>, rv: &mut HashMap<&'a Rc<ItemDef>, usize>) {
        match self {
            ItemCondition::NoRequirements => {},
            ItemCondition::Unattainable => {},
            ItemCondition::Item(item, count) => {
                if items.contains_key(item) {
                    let entry = rv.entry(item).or_default();
                    *entry = (*entry).max(*count);
                }
            },
            ItemCondition::AtLeast(threshold, req_item) => {
                let mut total = 0;
                // Reduce by contents of rv first
                for (item, weight) in req_item {
                    if let Some(&item_count) = rv.get(item) {
                        total += item_count * *weight;
                    }
                }
                if total >= *threshold {
                    return;
                }
                
                for (item, weight) in req_item {
                    if let Some(&item_count) = items.get(item) {
                        let mut item_count = item_count;
                        item_count -= rv.get(item).copied().unwrap_or(0);
                        total += item_count * *weight;
                        let entry = rv.entry(item).or_default();
                        *entry = (*entry).max(item_count);
                    }
                    if total >= *threshold {
                        break;
                    }
                }
            }
            ItemCondition::And(conds) => {
                for cond in conds {
                    cond.min_sat(items, rv);
                }
            },
            ItemCondition::Or(conds) => {
                let new_rv = conds.iter().map(|cond| {
                    let mut temp_rv = rv.clone();
                    cond.min_sat(items, &mut temp_rv);
                    temp_rv
                }).min_by_key(|temp_rv| {
                    let mut cost = 0;
                    for (item, count) in &*rv  {
                        cost += *count - temp_rv.get(item).copied().unwrap_or(0);
                    }
                    cost
                });
                if let Some(new_rv) = new_rv {
                    *rv = new_rv;
                }
            },
        }
    }

    pub fn prune_sat(&self, items: &HashMap<&Rc<ItemDef>, usize>) -> Result<ItemCondition, ()> {
        match self {
            ItemCondition::NoRequirements => Ok(self.clone()),
            ItemCondition::Unattainable => Err(()),
            ItemCondition::Item(item, count) => {
                if items.get(item).copied().unwrap_or(0) >= *count {
                    Ok(self.clone())
                } else {
                    Err(())
                }
            }
            ItemCondition::AtLeast(threshold, req_items) => {
                let mut total = 0;
                let mut rv: Vec<(Rc<ItemDef>, usize)> = vec![];
                for (req_item, weight) in req_items {
                    total += items.get(req_item).copied().unwrap_or(0) * *weight;
                    if total >= *threshold {
                        let diff = total - *threshold;
                        rv.push((req_item.clone(), *weight - (diff / weight)));
                        return Ok(ItemCondition::AtLeast(*threshold, rv))
                    } else {
                        rv.push((req_item.clone(), *weight));
                    }
                }
                Err(())
            }
            ItemCondition::And(conds) => {
                let conds = conds.iter().map(|c| c.prune_sat(items)).collect::<Result<_, _>>()?;
                Ok(ItemCondition::And(conds))
            },
            ItemCondition::Or(conds) => {
                let conds: Vec<_> = conds.iter().map(|c| c.prune_sat(items)).filter_map(|r| r.ok()).collect();
                match conds.len() {
                    0 => Ok(ItemCondition::Unattainable),
                    1 => Ok(conds[0].clone()),
                    _ => Ok(ItemCondition::Or(conds))
                }
            },
        }
    }
    pub fn would_be_satisfied_by(&self, item: &ItemDef) -> bool {
        match self {
            ItemCondition::NoRequirements => true,
            ItemCondition::Unattainable => false,
            ItemCondition::Item(cond_item, count) => item == &**cond_item && *count == 1,
            ItemCondition::AtLeast(threshold, req_items) => {
                req_items.iter().find(|(i, _)| &**i == item).map(|(_, weight)| *weight).unwrap_or(0) >= *threshold
            },
            ItemCondition::And(conds) => conds.iter().all(|cond| cond.would_be_satisfied_by(item)),
            ItemCondition::Or(conds) => conds.iter().any(|cond| cond.would_be_satisfied_by(item)),
        }
    }
    pub fn satisfied(&self) -> bool {
        self == &Self::NoRequirements
    }
    // pub fn satisfied(&self, items: &HashMap<Rc<ItemDef>, usize>) -> bool {
    //     match self {
    //         Self::NoRequirements => true,
    //         Self::Unattainable => false,
    //         Self::AtLeast(threshold, req_items) => {
    //             let mut total = 0;
    //             for (id, weight) in req_items {
    //                 total += items.get(id).copied().unwrap_or(0) * *weight;
    //             }

    //             total >= *threshold
    //         }
    //         Self::Item(id, count) => items.get(id).copied().unwrap_or(0) >= *count,
    //         Self::Or(conds) => conds.iter().any(|cond| cond.satisfied(items)),
    //         Self::And(conds) => conds.iter().all(|cond| cond.satisfied(items)),
    //     }
    // }

}

impl std::fmt::Display for ItemCondition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NoRequirements => write!(f, "-")?,
            Self::Unattainable => write!(f, "!")?,
            Self::Item(item, count) => {
                if *count > 1 {
                    write!(f, "{}*{}", item.name, count)?;
                } else {
                    write!(f, "{}", item.name)?;
                }
            }
            Self::AtLeast(threshold, items) => {
                write!(f, "({} <= {})", threshold, items.iter().map(|(item, weight)| {
                    if *weight > 1 {
                        format!("{} * {}", item, weight)
                    } else {
                        format!("{}", item)
                    }
                }).join_with(" + "))?;
            }
            Self::And(conds) => {
                write!(f, "({})", conds.iter().join_with(" & "))?;
            }
            Self::Or(conds) => {
                write!(f, "({})", conds.iter().join_with(" | "))?;
            },
        }
        Ok(())
    }
}