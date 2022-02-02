use std::{io::Error, path::Path, collections::{HashMap, HashSet}, hash::Hash, fmt::Write};
use heck::ToSnakeCase;
use rand::prelude::*;

pub fn allocate(cond_fact: &ConditionFactory, junk: &Item) {
    let mut assignable = vec![];
    let mut flags = HashSet::new();
    let mut locations = HashSet::new();
    let mut items = HashSet::new();
    let mut unassigned_items: HashSet<&Item> = cond_fact.items().collect();
    let mut assignments = HashMap::new();
    let mut graph = "digraph G {\n".to_string();
    loop {
        assignable.clear();
        for flag in cond_fact.flags() {
            if flag.requirement.satisfied(&items, &flags, &locations) {
                flags.insert(flag.id);
            }
        }
        let mut closed_locations = HashSet::new();
        for location in cond_fact.locations() {
            if location.requirement.satisfied(&items, &flags, &locations) {
                if !assignments.contains_key(location) {
                    assignable.push(location);
                }
                locations.insert(location.id);
            } else {
                closed_locations.insert(location);
            }
        }
        
        let mut items_unlock: HashMap<ItemId, Vec<&Location>> = HashMap::new();
        let mut items_unlock_flags: HashMap<ItemId, Vec<&Flag>> = HashMap::new();
        for item in &unassigned_items {
            if items.insert(item.id) {
                for location in &closed_locations {
                    if location.requirement.satisfied(&items, &flags, &locations) {
                        items_unlock.entry(item.id).or_default().push(location);
                    }
                }
                // for flag in cond_fact.flags() {
                //     if flags.contains(&flag.id) {
                //         continue;
                //     }
                //     if flag.requirement.satisfied(&items, &flags, &locations) {
                //         items_unlock_flags.entry(item.id).or_default().push(flag);
                //     }
                // }
                items.remove(&item.id);
            }        
        }
        for (item, locs) in &items_unlock {
            let item = cond_fact.get_item(*item).unwrap();
            // println!("{}:", item.name);
            // for loc in locs {
            //     println!("  {}", loc.name);
            // }
            // println!();

        }
        for (item, locs) in &items_unlock_flags {
            let item = cond_fact.get_item(*item).unwrap();
            // println!("{}:", item.name);
            // for loc in locs {
            //     println!("  {}", loc.name);
            // }
            // println!();

        }
        // if let Some((next_item, _flags)) = items_unlock_flags.iter().next() {
            
        //     items.insert(*next_item);
        //     let location = assignable.pop().expect("No where left to place critical item!");
        //     println!("Unlocking flag");
        //     assignments.insert(location, cond_fact.get_item(*next_item).unwrap());
        //     let item = cond_fact.get_item(*next_item).unwrap();
        //     unassigned_items.remove(item);
        //     continue;
        // }

        if let Some((next_item, unlocked_locations)) = items_unlock.iter().next() {
            
            items.insert(*next_item);
            let location = assignable.pop().expect("No where left to place critical item!");
            let item = cond_fact.get_item(*next_item).unwrap();
            let satisfiers = location.requirement.satisfied_by(cond_fact, &items, &flags, &locations);
            graph.push_str(&format!(r#"  "{}" [label="{}\n{}"];
"#, item.name, location.name, item.name));
            for satisfier in satisfiers {
                let sat_item = cond_fact.get_item(satisfier).unwrap();
                graph.push_str(&format!("  \"{}\" -> \"{}\";\n", item.name, sat_item.name));
            }
            assignments.insert(location, item);
            println!(" + {} {} {}", assignable.len() + 1, unlocked_locations.len(), closed_locations.len());

            if closed_locations.len() > unassigned_items.iter().filter(|i| i.category == ItemCategory::Major).count() {
                for loc in assignable.drain(..) {
                    assignments.insert(loc, junk);
                }
            }
            
            // Pick an location to insert the item in
            for location in unlocked_locations {
                closed_locations.remove(location);
                assignable.push(location);
            }
            let item = cond_fact.get_item(*next_item).unwrap();
            unassigned_items.remove(item);
            continue;
        }
        
        println!(" | {} {}", assignable.len(), unassigned_items.iter().filter(|i| i.category == ItemCategory::Major).count());
        println!("Closed locations: ");
        for loc in closed_locations {
            println!("  {}:", loc.name);
            print!("    ");
            let mut reqs = loc.requirement.expand(cond_fact).flatten().simplify();
            for item in &items {
                reqs.assume_item(*item);
            }
            reqs.simplify();
            reqs.render(cond_fact);
            println!();
        }
        break;
        // Pick a random item and place it in a random location
        if let Some(&item_to_allocate) = unassigned_items.iter().filter(|i| i.category == ItemCategory::Major).choose(&mut thread_rng()) {
            println!("Trying to find a home for {}", item_to_allocate.name);
            if let Some(loc) = assignable.choose(&mut thread_rng()) {
                items.insert(item_to_allocate.id);
                assignments.insert(loc, item_to_allocate);
                unassigned_items.remove(item_to_allocate);
                continue;
            } else {   
                panic!("Unallocated items remain, but no spaces are available");
            }
            
        }

        println!();
        if items_unlock.is_empty() && items_unlock_flags.is_empty() {
            break;
        }
        
    }
    graph.push_str(r#"  "Beat Game" [shape="box"];
    "Beat Game" -> "FourSword";
    "Beat Game" -> "BombBag";
    "Beat Game" -> "Bow";
    "Beat Game" -> "RocsCape";
    "Beat Game" -> "LanternOff";
    "Beat Game" -> "GustJar";
    "Beat Game" -> "PacciCane";
    "#);
    graph.push_str("}\n");
    std::fs::write("graph.dot", graph.as_bytes()).unwrap();
    for (location, item) in assignments {
        println!("{} -> {}", location.name, item.name);
    //     print!("  ");
    //     location.requirement.expand(cond_fact).render(cond_fact);
    //     println!();
    //     print!("  ");
    //     let loc_cond = location.requirement.expand(cond_fact).flatten();        
    //     loc_cond.render(cond_fact);
    //     println!();
    //     print!("  ");
    //     let loc_cond = loc_cond.simplify();
    //     loc_cond.render(cond_fact);
    //     println!();
    //     println!();
    // }
    // for loc in assignable {
    //     print!("{}, ", loc.name);
    }
    println!();
    println!("Unassigned: ");
    for item in unassigned_items {
        print!("{}, ", item.name);
    }
}

pub struct ConditionFactory {
    last_id: usize,
    item_map: HashMap<ItemId, Item>,
    flag_map: HashMap<FlagId, Flag>,
    location_map: HashMap<LocationId, Location>,
}
impl ConditionFactory {
    pub fn new() -> Self {
        ConditionFactory { last_id: 0, item_map: HashMap::new(), location_map: HashMap::new(), flag_map: HashMap::new() }
    }
    pub fn graph(&self) -> String {
        unimplemented!()
    }
    pub fn flags(&self) -> impl Iterator<Item=&Flag> {
        self.flag_map.values()
    }
    pub fn items(&self) -> impl Iterator<Item=&Item> {
        self.item_map.values()
    }
    pub fn locations(&self) -> impl Iterator<Item=&Location> {
        self.location_map.values()
    }
    pub fn get_location(&self, id: LocationId) -> Option<&Location> {
        self.location_map.get(&id)
    }
    pub fn get_item(&self, id: ItemId) -> Option<&Item> {
        self.item_map.get(&id)
    }
    pub fn get_flag(&self, id: FlagId) -> Option<&Flag> {
        self.flag_map.get(&id)
    }
    pub fn add_item(&mut self, name: impl Into<String>, category: ItemCategory) -> ItemId {
        self.last_id += 1;
        let name = name.into();
        let id = ItemId(self.last_id);
        let item = Item {
            name,
            id,
            category
        };
        self.item_map.insert(id, item);
        id
    }
    pub fn add_location(&mut self, name: impl Into<String>, requirement: impl Into<Condition>) -> LocationId {
        let name = name.into();
        let requirement= requirement.into().expand(self).flatten().simplify();
        self.last_id += 1;
        let id = LocationId(self.last_id);
        let location = Location {
            name,
            requirement,
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
    Item(ItemId),
    Location(LocationId),
    And(Vec<Condition>),
    Or(Vec<Condition>),
    Not(Box<Condition>)
}

impl Condition {
    pub fn expand(&self, cond_fact: &ConditionFactory) -> Condition {
        match self {
            Condition::NoRequirements => Condition::NoRequirements,
            Condition::Flag(id) => {
                let flag = cond_fact.get_flag(*id).unwrap();
                flag.requirement.expand(cond_fact)
            },
            Condition::Item(_) => self.clone(),
            Condition::Location(id) => {
                let location = cond_fact.get_location(*id).unwrap();
                location.requirement.expand(cond_fact)
            }
            Condition::And(conds) => {
                Condition::And(conds.iter().map(|c| c.expand(cond_fact)).collect())
            },
            Condition::Or(conds) => Condition::Or(conds.iter().map(|c| c.expand(cond_fact)).collect()),
            Condition::Not(_) => todo!(),
        }
    }
    fn top_level_items(&self) -> HashSet<ItemId> {
        match self {
            Condition::NoRequirements | Condition::Flag(_) | Condition::Location(_) => HashSet::new(),
            Condition::Item(id) => {
                let mut rv = HashSet::with_capacity(1);
                rv.insert(*id);
                rv
            },
            Condition::And(conds) => conds.iter().filter_map(|c| match c { Condition::Item(id) => Some(*id), _ => None }).collect(),
            Condition::Or(conds) => conds.iter().filter_map(|c| match c { Condition::Item(id) => Some(*id), _ => None }).collect(),
            Condition::Not(_) => todo!(),
        }
    }
    pub fn flatten(&self) -> Condition {
        match self {
            Condition::NoRequirements => self.clone(),
            Condition::Flag(_) | Condition::Location(_) => panic!(),
            Condition::Item(_) => self.clone(),
            Condition::And(conds) => {
                let mut new_conds = vec![];
                for cond in conds {
                    let new_cond = cond.flatten();
                    match new_cond {
                        Condition::And(conds) => new_conds.extend(conds),
                        _ => new_conds.push(new_cond)
                    }
                }
                new_conds.sort();
                new_conds.dedup();
                Condition::And(new_conds)
            },
            Condition::Or(conds) => {
                let mut new_conds = vec![];
                for cond in conds {
                    let new_cond = cond.flatten();
                    match new_cond {
                        Condition::Or(conds) => new_conds.extend(conds),
                        _ => new_conds.push(new_cond)
                    }
                }
                new_conds.sort();
                new_conds.dedup();
                Condition::Or(new_conds)
            },
            Condition::Not(_) => todo!(),
        }
    }
    fn remove_redundant_ors(&mut self, or: &[Condition]) -> bool {
        let mut changed = false;
        match self {
            Condition::And(conds) => {
                for cond in &mut *conds {
                    changed |= cond.remove_redundant_ors(or);
                }
                if changed {
                    conds.retain(|c| *c != Condition::NoRequirements);
                    match conds.len() {
                        0 => *self = Condition::NoRequirements,
                        1 => *self = conds.pop().expect("Looked before we lept"),
                        _ => {}
                    }
                }
            },
            Condition::Or(conds) => {
                if or.iter().all(|c| conds.contains(c)) {
                    *self = Condition::NoRequirements;
                    changed = true;
                } else {
                    for cond in &mut *conds {
                        changed |= cond.remove_redundant_ors(or);
                    }
                    if changed {
                        conds.retain(|c| *c != Condition::NoRequirements);
                        match conds.len() {
                            0 => *self = Condition::NoRequirements,
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
    pub fn simplify(&self) -> Condition {
        match self {
            Condition::NoRequirements => self.clone(),
            Condition::Flag(_) | Condition::Location(_) => panic!(),
            Condition::Item(_) => self.clone(),
            Condition::And(conds) => {
                let mut new_conds = vec![];
                for cond in conds {
                    let new_cond = cond.flatten();
                    match new_cond {
                        Condition::And(conds) => new_conds.extend(conds),
                        _ => new_conds.push(new_cond)
                    }
                }
                let items: Vec<_> = new_conds.iter().filter_map(|c| match c { Condition::Item(id) => Some(*id), _ => None }).collect();
                for item in items {
                    for cond in &mut new_conds {
                        if matches!(cond, Condition::Or(_) | Condition::And(_)) {
                            cond.assume_item(item);
                        }
                    }
                }
                new_conds.retain(|c| *c != Condition::NoRequirements);
                new_conds.sort();
                new_conds.dedup();
                let disj: Vec<_> = new_conds.iter().filter_map(|c| match c { Condition::Or(conds) => Some(conds.clone()), _ => None }).collect();
                for cond in &mut new_conds {
                    if matches!(cond, Condition::And(_) | Condition::Or(_)) {
                        for or in &disj {
                            if matches!(cond, Condition::Or(cond) if cond == or) {
                                continue;
                            }
                            cond.remove_redundant_ors(or);
                        }
                    }                    
                }
                new_conds.retain(|c| *c != Condition::NoRequirements);
                new_conds.sort();
                new_conds.dedup();
                
                let rv = Condition::And(new_conds);
                rv
            },
            Condition::Or(conds) => {
                let mut conds = conds.clone();
                conds.sort();
                conds.dedup();
                Condition::Or(conds)
            },
            Condition::Not(_) => todo!(),
        }
    }
    pub fn assume_item(&mut self, id: ItemId) {
        match self {
            Condition::NoRequirements | Condition::Flag(_) | Condition::Location(_) => {}
            Condition::Item(my_id) => if id == *my_id {
                *self = Condition::NoRequirements;
            },
            Condition::And(conds) => {
                for cond in &mut *conds {
                    cond.assume_item(id);
                }
                conds.retain(|c| *c != Condition::NoRequirements);
                if conds.len() == 0 {
                    *self = Condition::NoRequirements;
                } else if conds.len() == 1 {
                    *self = conds.pop().expect("We just checked that conds is non-empty");
                }
            }
            Condition::Or(conds) => {
                for cond in &mut *conds {
                    cond.assume_item(id);
                }
                if conds.iter().any(|c| *c == Condition::NoRequirements) {
                    *self = Condition::NoRequirements;
                }
            }
            Condition::Not(_) => todo!(),
        }
    }

    pub fn render(&self, cond_fact: &ConditionFactory) {
        match self {
            Condition::NoRequirements => print!("-"),
            Condition::Flag(id) => {
                let flag = cond_fact.get_flag(*id).unwrap();
                // print!("*{}", flag.name);
                flag.requirement.render(cond_fact);
            }
            Condition::Item(id) => {
                let item = cond_fact.get_item(*id).unwrap();
                print!("{}", item.name);
            }
            Condition::Location(id) => {
                let location = cond_fact.get_location(*id).unwrap();
                // print!("!{}", location.name);
                location.requirement.render(cond_fact);
            }
            Condition::And(conds) => {
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
            Condition::Or(conds) => {
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
            Condition::Not(_) => todo!(),
        }
    }
    pub fn satisfied(&self, items: &HashSet<ItemId>, flags: &HashSet<FlagId>, locations: &HashSet<LocationId>) -> bool {
        match self {
            Condition::NoRequirements => true,
            Condition::Flag(id) => flags.contains(id),
            Condition::Location(id) => locations.contains(id),
            Condition::Item(id) => items.contains(id),
            Condition::Not(cond) => !cond.satisfied(items, flags, locations),
            Condition::Or(conds) => conds.iter().any(|cond| cond.satisfied(items, flags, locations)),
            Condition::And(conds) => conds.iter().all(|cond| cond.satisfied(items, flags, locations)),
        }
    }

    pub fn satisfied_by(&self, cond_fact: &ConditionFactory, items: &HashSet<ItemId>, flags: &HashSet<FlagId>, locations: &HashSet<LocationId>) -> HashSet<ItemId> {
        match self {
            Condition::NoRequirements => HashSet::new(),
            Condition::Flag(id) => {
                if let Some(flag) = cond_fact.get_flag(*id) {
                    flag.requirement.satisfied_by(cond_fact, items, flags, locations)
                } else {
                    panic!();
                }
            },
            Condition::Item(id) => {
                let mut rv = HashSet::new();
                if items.contains(id) {
                    rv.insert(*id);
                }
                rv
            }
            Condition::Location(id) => if let Some(location) = cond_fact.get_location(*id) {
                location.requirement.satisfied_by(cond_fact, items, flags, locations)
            } else {
                panic!();
            }
            Condition::And(conds) => {
                let mut rv = HashSet::new();
                for cond in conds {
                    rv.extend(cond.satisfied_by(cond_fact, items, flags, locations));
                }
                rv
            },
            Condition::Or(conds) => {
                for cond in conds {
                    if cond.satisfied(items, flags, locations) {
                        return cond.satisfied_by(cond_fact, items, flags, locations);
                    }
                }
                HashSet::new()
            },
            Condition::Not(_) => todo!(),
        }
    }
}

impl From<FlagId> for Condition {
    fn from(id: FlagId) -> Self {
        Condition::Flag(id)
    }
}

impl From<ItemId> for Condition {
    fn from(id: ItemId) -> Self {
        Condition::Item(id)
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
    requirement: Condition
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ItemCategory {
    Major,
    Minor,
    DungeonItem
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Item {
    name: String,
    id: ItemId,
    category: ItemCategory
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
    requirement: Condition
}