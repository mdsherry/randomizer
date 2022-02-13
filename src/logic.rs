use serde::Deserialize;
use std::{collections::HashMap, rc::Rc};

mod condition;
pub use condition::*;
mod allocator;
pub use allocator::*;
mod logic_loader;
pub use logic_loader::*;
mod allocation_checker;
pub use allocation_checker::*;
pub struct PreLogic {
    last_id: usize,
    parameters: HashMap<String, bool>,
    item_map: HashMap<ItemId, PreItemDef>,
    flag_map: HashMap<FlagId, PreFlag>,
    location_map: HashMap<LocationId, PreLocation>,
}

pub struct Logic {
    pub items: Vec<(Rc<ItemDef>, ItemCondition)>,
    pub flags: Vec<(Rc<Flag>, ItemCondition)>,
    pub locations: Vec<(Rc<Location>, ItemCondition)>,
}

impl PreLogic {
    pub fn new() -> Self {
        PreLogic {
            last_id: 0,
            parameters: Default::default(),
            item_map: HashMap::new(),
            location_map: HashMap::new(),
            flag_map: HashMap::new(),
        }
    }

    pub fn set_parameter(&mut self, name: impl Into<String>, value: bool) {
        self.parameters.insert(name.into(), value);
    }
    pub fn get_parameter(&self, name: &str) -> bool {
        self.parameters.get(name).copied().unwrap_or_default()
    }
    pub fn build(&self, item_pool_ids: &[ItemId]) -> (Logic, Vec<(Rc<ItemDef>, ItemCondition)>) {
        let mut condition_cache: HashMap<&Condition, ItemCondition> = HashMap::new();
        let mut item_cache = HashMap::new();
        let mut items = vec![];
        for preitem in self.item_map.values() {
            let item = Rc::new(ItemDef {
                name: preitem.name.clone(),
                category: preitem.category,
                restriction: preitem.restriction,
                weight: preitem.weight,
                show_in_graph: preitem.show_in_graph,
            });
            items.push(item.clone());
            item_cache.insert(preitem.id, item);
        }

        let mut items = vec![];
        for preitem in self.item_map.values() {
            let item = item_cache[&preitem.id].clone();
            items.push((
                item,
                preitem
                    .requirement
                    .expand(&self, &mut condition_cache, &item_cache),
            ))
        }
        let mut flags = vec![];
        for flag in self.flag_map.values() {
            flags.push((
                Rc::new(Flag {
                    name: flag.name.clone(),
                }),
                flag.requirement
                    .expand(&self, &mut condition_cache, &item_cache),
            ));
        }

        let mut locations = vec![];
        for location in self.location_map.values() {
            locations.push((
                Rc::new(Location {
                    name: location.name.clone(),
                    category: location.category,
                    restriction: location.restriction,
                }),
                location
                    .requirement
                    .expand(&self, &mut condition_cache, &item_cache),
            ));
        }
        let logic = Logic {
            items,
            flags,
            locations,
        };
        let item_pool = item_pool_ids
            .iter()
            .map(|id| {
                let item = item_cache[id].clone();
                let cond = condition_cache[&self.item_map[id].requirement].clone();
                (item, cond)
            })
            .collect();
        (logic, item_pool)
    }
    pub fn get_location(&self, id: LocationId) -> Option<&PreLocation> {
        self.location_map.get(&id)
    }
    pub fn get_item(&self, id: ItemId) -> Option<&PreItemDef> {
        self.item_map.get(&id)
    }
    pub fn get_flag(&self, id: FlagId) -> Option<&PreFlag> {
        self.flag_map.get(&id)
    }
    pub fn add_item(
        &mut self,
        name: impl Into<String>,
        category: ItemCategory,
        restriction: Option<Restriction>,
        weight: Option<u32>,
        show_in_graph: bool,
    ) -> ItemId {
        self.last_id += 1;
        let name = name.into();
        let id = ItemId(self.last_id);
        let item = PreItemDef {
            name,
            id,
            category,
            weight: weight.unwrap_or(1),
            restriction,
            requirement: Condition::NoRequirements,
            show_in_graph,
        };
        self.item_map.insert(id, item);
        id
    }
    pub fn add_item_requirement(&mut self, id: ItemId, requirement: Condition) {
        if let Some(item) = self.item_map.get_mut(&id) {
            item.requirement = requirement;
        }
    }
    pub fn add_flag_requirement(&mut self, id: FlagId, requirement: Condition) {
        if let Some(flag) = self.flag_map.get_mut(&id) {
            flag.requirement = requirement;
        }
    }
    pub fn add_location_requirement(&mut self, id: LocationId, requirement: Condition) {
        if let Some(location) = self.location_map.get_mut(&id) {
            location.requirement = requirement;
        }
    }
    pub fn add_location(
        &mut self,
        name: impl Into<String>,
        category: ItemCategory,
        restriction: Option<Restriction>,
    ) -> LocationId {
        let name = name.into();
        self.last_id += 1;
        let id = LocationId(self.last_id);
        let location = PreLocation {
            name,
            requirement: Condition::NoRequirements,
            restriction,
            category,
            id,
        };
        self.location_map.insert(id, location);
        id
    }

    pub fn add_flag(&mut self, name: impl Into<String>) -> FlagId {
        let name = name.into();

        self.last_id += 1;
        let id = FlagId(self.last_id);
        let flag = PreFlag {
            name,
            requirement: Condition::NoRequirements,
            id,
        };
        self.flag_map.insert(id, flag);
        id
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct PreFlag {
    name: String,
    id: FlagId,
    requirement: Condition,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Flag {
    name: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Deserialize)]
pub enum ItemCategory {
    Minor,
    Major,
    DungeonItem,
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
pub struct PreItemDef {
    pub name: String,
    pub id: ItemId,
    pub category: ItemCategory,
    pub restriction: Option<Restriction>,
    pub weight: u32,
    pub show_in_graph: bool,
    pub requirement: Condition,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ItemDef {
    pub name: String,
    pub category: ItemCategory,
    pub restriction: Option<Restriction>,
    pub weight: u32,
    pub show_in_graph: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Item {
    pub def: Rc<ItemDef>,
    pub count: usize,
}
impl std::fmt::Display for ItemDef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}", self.category.decoration(), self.name)?;
        if let Some(r) = self.restriction {
            write!(f, "{}", r)?;
        }
        Ok(())
    }
}
impl Item {
    pub fn new(def: Rc<ItemDef>) -> Self {
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
pub struct PreLocation {
    name: String,
    id: LocationId,
    category: ItemCategory,
    requirement: Condition,
    restriction: Option<Restriction>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Location {
    name: String,
    category: ItemCategory,
    restriction: Option<Restriction>,
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
