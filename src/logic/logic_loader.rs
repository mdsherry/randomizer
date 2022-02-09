use std::{io::Read, collections::HashMap};

use serde::Deserialize;

use crate::logic_parse::{parse_reqs, gen_reqs2};

use super::{ItemId, ItemCategory, Logic, Restriction};

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
    pub fn from_reader(reader: impl Read) -> (Logic, Vec<ItemId>) {
        let mut logic = Logic::new();
        let data: LogicData = serde_yaml::from_reader(reader).unwrap();
        let mut items = HashMap::new();
        let mut flags = HashMap::new();
        let mut locations = HashMap::new();
        let mut restrictions = HashMap::new();
        let mut restriction_id = 0;
        for item in &data.item_pool {
            let restriction = item.restriction.as_ref().map(|restriction| restrictions.entry(restriction).or_insert_with(|| {
                restriction_id += 1;
                Restriction(restriction_id)
            })).copied();
            let id = logic.add_item(&item.name, item.category, restriction, item.weight, item.show_in_graph.unwrap_or(false));
            items.insert(item.name.as_str(), id);
        }
        for flag in &data.flags {
            let reqs = parse_reqs(flag.requirements.as_deref().unwrap_or(""));
            let conditions = gen_reqs2(&reqs.0, &items, &flags, &locations).map_err(|e| {
                eprintln!("Error parsing flag {}", flag.name);
                panic!("{}", e);
            }).unwrap();
            let id = logic.add_flag(flag.name.as_str(), conditions);
            flags.insert(flag.name.as_str(), id);
        }
        for location in &data.locations {
            let reqs = parse_reqs(location.requirements.as_deref().unwrap_or(""));
            let conditions = gen_reqs2(&reqs.0, &items, &flags, &locations).map_err(|e| {
                eprintln!("Error parsing location {}", location.name);
                panic!("{}", e);
            }).unwrap();
            let restriction = location.restriction.as_ref().map(|restriction| restrictions.entry(restriction).or_insert_with(|| {
                restriction_id += 1;
                Restriction(restriction_id)
            })).copied();
            let id = logic.add_location(location.name.as_str(), conditions, location.category, restriction); // FIX
            locations.insert(location.name.as_str(), id);
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