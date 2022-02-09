use std::{collections::HashMap, io::Read};
use serde::Deserialize;
use crate::logic_parse::{parse_reqs, gen_reqs2};

mod condition;
pub use condition::*;
mod allocator;
pub use allocator::*;
mod logic_loader;
pub use logic_loader::*;

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
    pub fn add_item(&mut self, name: impl Into<String>, category: ItemCategory, restriction: Option<Restriction>, weight: Option<u32>, show_in_graph: bool) -> ItemId {
        self.last_id += 1;
        let name = name.into();
        let id = ItemId(self.last_id);
        let item = ItemDef {
            name,
            id,
            category,
            weight: weight.unwrap_or(1),
            restriction,
            show_in_graph
        };
        self.item_map.insert(id, item);
        id
    }
    pub fn add_location(&mut self, name: impl Into<String>, requirement: impl Into<Condition>, category: ItemCategory, restriction: Option<Restriction>) -> LocationId {
        let name = name.into();
        let requirement= requirement.into().expand(self).flatten().simplify();
        self.last_id += 1;
        let id = LocationId(self.last_id);
        let location = Location {
            name,
            requirement,
            restriction,
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


impl From<FlagId> for Condition {
    fn from(id: FlagId) -> Self {
        Condition::Flag(id)
    }
}

impl From<ItemId> for Condition {
    fn from(id: ItemId) -> Self {
        Condition::Item(id, 1)
    }
}
impl From<LocationId> for Condition {
    fn from(id: LocationId) -> Self {
        Condition::Location(id)
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Flag {
    name: String,
    id: FlagId,
    requirement: ItemCondition
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Deserialize)]
pub enum ItemCategory {
    Minor,
    Major,
    DungeonItem
}
impl ItemCategory {
    pub fn decoration(self) -> &'static str {
        match self {
            ItemCategory::Minor => " ",
            ItemCategory::Major => "!",
            ItemCategory::DungeonItem => "*",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ItemDef {
    pub name: String,
    pub id: ItemId,
    pub category: ItemCategory,
    pub restriction: Option<Restriction>,
    pub weight: u32,
    pub show_in_graph: bool
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Item<'a> {
    pub def: &'a ItemDef,
    pub count: usize
}
impl<'a> std::fmt::Display for Item<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}", self.def.category.decoration(), self.def.name)?;
        if let Some(r) = self.def.restriction {
            write!(f, "{}", r)?;
        }
        Ok(())
    }
}
impl<'a> Item<'a> {
    pub fn new(def: &'a ItemDef) -> Self {
        Item { def, count: 1 }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Restriction(usize);
impl<'a> std::fmt::Display for Restriction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}]", self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FlagId(usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ItemId(usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct LocationId(usize);

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Location {
    name: String,
    id: LocationId,
    category: ItemCategory,
    requirement: ItemCondition,
    restriction: Option<Restriction>
}
impl std::fmt::Display for Location {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}", self.category.decoration(), self.name)?;
        if let Some(r) = self.restriction {
            write!(f, "{}", r)?;
        }
        Ok(())
    }
}