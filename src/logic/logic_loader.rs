use std::{io::Read, collections::HashMap};

use serde::Deserialize;

use crate::logic_parse::{parse_reqs, gen_reqs2};

use super::{ItemId, ItemCategory, PreLogic, Restriction};

pub struct LogicLoader;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all="snake_case")]
struct LogicData {
    flags: Vec<FlagData>,
    item_pool: Vec<ItemData>,
    locations: Vec<LocationData>
}
#[derive(Debug, Clone, Deserialize)]
struct ItemData {
    name: String,
    category: ItemCategory,
    restriction: Option<String>,
    count: Option<u32>,
    weight: Option<u32>,
    requirements: Option<String>,
    show_in_graph: Option<bool>
}
#[derive(Debug, Clone, Deserialize)]
struct FlagData {
    name: String,
    requirements: Option<String>
}
#[derive(Debug, Clone, Deserialize)]
struct LocationData {
    name: String,
    category: ItemCategory,
    requirements: Option<String>,
    restriction: Option<String>,    
}

impl LogicLoader {
    pub fn from_reader(reader: impl Read) -> (PreLogic, Vec<ItemId>) {
        let mut logic = PreLogic::new();
        let data: LogicData = serde_yaml::from_reader(reader).unwrap();
        let mut items = HashMap::new();
        let mut flags = HashMap::new();
        let mut locations = HashMap::new();
        let mut restrictions = HashMap::new();
        let mut restriction_id = 0;
        let mut get_restriction = |name: &str| {
            *restrictions.entry(name.to_string()).or_insert_with(|| {
                restriction_id += 1;
                Restriction(restriction_id)
            })
        };
        for item in &data.item_pool {
            let restriction = item.restriction.as_deref().map(&mut get_restriction);
            let id = logic.add_item(&item.name, item.category, restriction, item.weight, item.show_in_graph.unwrap_or(false));
            items.insert(item.name.as_str(), id);
        }
        for flag in &data.flags {
            let id = logic.add_flag(flag.name.as_str());
            flags.insert(flag.name.as_str(), id);
        }
        for location in &data.locations {
            let restriction = location.restriction.as_deref().map(&mut get_restriction);
            let id = logic.add_location(location.name.as_str(), location.category, restriction);
            locations.insert(location.name.as_str(), id);
        }
        for item in &data.item_pool {
            let id = items[item.name.as_str()];
            let (reqs, _) = parse_reqs(item.requirements.as_deref().unwrap_or(""));
            let conditions = gen_reqs2(&reqs, &items, &flags, &locations).map_err(|e| {
                eprintln!("Error parsing item {}", item.name);
                panic!("{}", e);
            }).unwrap();
            logic.add_item_requirement(id, conditions);
        }
        for flag in &data.flags {
            let id = flags[flag.name.as_str()];
            let (reqs, _) = parse_reqs(flag.requirements.as_deref().unwrap_or(""));
            let conditions = gen_reqs2(&reqs, &items, &flags, &locations).map_err(|e| {
                eprintln!("Error parsing flag {}", flag.name);
                panic!("{}", e);
            }).unwrap();
            logic.add_flag_requirement(id, conditions);
        }
        for location in &data.locations {
            let id = locations[location.name.as_str()];
            let (reqs, _) = parse_reqs(location.requirements.as_deref().unwrap_or(""));
            let conditions = gen_reqs2(&reqs, &items, &flags, &locations).map_err(|e| {
                eprintln!("Error parsing location {}", location.name);
                panic!("{}", e);
            }).unwrap();
            logic.add_location_requirement(id, conditions);
        }
        
        let mut item_pool = vec![];
        for item in &data.item_pool {
            let id = *items.get(item.name.as_str()).unwrap();
            for _ in 0..item.count.unwrap_or(1) {
                item_pool.push(id);
            }
        }
        println!("{:#?}", restrictions);
        (logic, item_pool)
    }
}