use std::collections::{HashMap, HashSet};

use super::{ItemId, FlagId, Flag, LocationId, Location, ItemCategory, ItemDef};

pub struct Logic {
    last_id: usize,
    item_map: HashMap<ItemId, ItemDef>,
    flag_map: HashMap<FlagId, Flag>,
    location_map: HashMap<LocationId, Location>,
}
impl Logic {
    pub fn new() -> Self {
        Logic { last_id: 0, item_map: HashMap::new(), location_map: HashMap::new(), flag_map: HashMap::new() }
    }
    pub fn graph(&self) -> String {
        unimplemented!()
    }
    pub fn flags(&self) -> impl Iterator<Item=&Flag> {
        self.flag_map.values()
    }
    pub fn items(&self) -> impl Iterator<Item=&ItemDef> {
        self.item_map.values()
    }
    pub fn locations(&self) -> impl Iterator<Item=&Location> {
        self.location_map.values()
    }
    pub fn get_location(&self, id: LocationId) -> Option<&Location> {
        self.location_map.get(&id)
    }
    pub fn get_item(&self, id: ItemId) -> Option<&ItemDef> {
        self.item_map.get(&id)
    }
    pub fn get_flag(&self, id: FlagId) -> Option<&Flag> {
        self.flag_map.get(&id)
    }
    pub fn add_item(&mut self, name: impl Into<String>, category: ItemCategory) -> ItemId {
        self.last_id += 1;
        let name = name.into();
        let id = ItemId(self.last_id);
        let item = ItemDef {
            name,
            id,
            category
        };
        self.item_map.insert(id, item);
        id
    }
    pub fn add_location(&mut self, name: impl Into<String>, requirement: impl Into<Condition>, category: ItemCategory) -> LocationId {
        let name = name.into();
        let requirement= requirement.into().expand(self).flatten().simplify();
        self.last_id += 1;
        let id = LocationId(self.last_id);
        let location = Location {
            name,
            requirement,
            category,
            id
        };
        self.location_map.insert(id, location);
        id
    }
    
    pub fn add_flag(&mut self, name: impl Into<String>, requirement: impl Into<Condition>) -> FlagId {
        let name = name.into();
        let requirement= requirement.into().expand(self).flatten().simplify();
        self.last_id += 1;
        let id = FlagId(self.last_id);
        let flag = Flag {
            name,
            requirement,
            id
        };
        self.flag_map.insert(id, flag);
        id
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Condition {
    NoRequirements,
    Flag(FlagId),
    Item(ItemId, usize),
    Location(LocationId),
    And(Vec<Self>),
    Or(Vec<Self>),
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ItemCondition {
    NoRequirements,
    Item(ItemId, usize),
    And(Vec<Self>),
    Or(Vec<Self>),
}


impl Condition {
    pub fn expand(&self, cond_fact: &Logic) -> ItemCondition {
        match self {
            Condition::NoRequirements => ItemCondition::NoRequirements,
            Condition::Flag(id) => {
                let flag = cond_fact.get_flag(*id).unwrap();
                flag.requirement.clone()
            },
            Condition::Item(id, count) => ItemCondition::Item(*id, *count),
            Condition::Location(id) => {
                let location = cond_fact.get_location(*id).unwrap();
                location.requirement.clone()
            }
            Condition::And(conds) => {
                ItemCondition::And(conds.iter().map(|c| c.expand(cond_fact)).collect())
            },
            Condition::Or(conds) => ItemCondition::Or(conds.iter().map(|c| c.expand(cond_fact)).collect()),
        }
    }
}
impl ItemCondition {
    pub fn complexity(&self) -> u32 {
        match self {
            Self::NoRequirements => 0,
            Self::Item(_, _) => 1,
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
            Self::And(conds) => conds.iter().filter_map(|c| match c { Self::Item(id, _) => Some(*id), _ => None }).collect(),
            Self::Or(conds) => conds.iter().filter_map(|c| match c { Self::Item(id, _) => Some(*id), _ => None }).collect(),
        }
    }
    pub fn missing(&self, items: &HashMap<ItemId, usize>, rv: &mut HashMap<ItemId, usize>) {
        if !self.satisfied(items) {
            match self {
                ItemCondition::NoRequirements => {},
                ItemCondition::Item(id, count) => {rv.insert(*id, *count);}
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
            // Condition::Flag(id) => flags.contains(id),
            // Condition::Location(id) => locations.contains(id),
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
