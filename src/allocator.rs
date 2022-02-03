mod condition;

use std::{collections::{HashMap, HashSet}, hash::Hash};
use rand::prelude::*;
pub use condition::*;


pub struct Allocator<'a> {
    logic: &'a Logic,
    item_pool: Vec<Item<'a>>,
    open_locations: Vec<&'a Location>,
    closed_locations: Vec<&'a Location>,

    assigned_items: HashMap<ItemId, usize>,
    items_by_class: HashMap<u32, usize>,
    locs_by_class: HashMap<u32, usize>,

    assignments: HashMap<&'a Location, Item<'a>>,
    graph: String
}

impl<'a> Allocator<'a> {
    pub fn new(logic: &'a Logic, item_pool: Vec<Item<'a>>) -> Self {
        let mut me = Allocator { 
            logic,
            item_pool,
            open_locations: Default::default(),
            closed_locations: Default::default(), 
            assigned_items: Default::default(),
            items_by_class: Default::default(),
            locs_by_class: Default::default(),
            assignments: Default::default(),
            graph: Default::default()
        };
        me.find_open_locs();
        me.compute_locs_by_class();
        println!("{:#?}", me.items_by_class);
        me
    }

    fn compute_locs_by_class(&mut self) {
        for item in &self.item_pool {
            if let ItemCategory::Class(n) = item.def.category {
                *self.items_by_class.entry(n).or_insert(0) += 1;
            }
        }
        for loc in &self.open_locations {
            if let ItemCategory::Class(n) = loc.category {
                *self.locs_by_class.entry(n).or_insert(0) += 1;
            }
        }
        
    }

    fn find_open_locs(&mut self) {
        for location in self.logic.locations() {
            if location.requirement.satisfied(&self.assigned_items) {
                if !self.assignments.contains_key(location) {
                    self.open_locations.push(location);
                }
            } else {
                self.closed_locations.push(location);
            }
        }
    }

    fn single_item_location_unlocks(&self) -> HashMap<Item<'a>, Vec<&'a Location>> {
        let mut assigned_items = self.assigned_items.clone();
        let mut things: HashMap<Item, Vec<_>> = HashMap::new();
        for item in &self.item_pool {
            *assigned_items.entry(item.def.id).or_default() += 1;
            for loc in &self.closed_locations {
                if loc.requirement.satisfied(&assigned_items) {
                    let entry = things.entry(*item).or_default();
                    entry.push(*loc);
                    entry.sort_unstable_by_key(|l| l.id);
                    entry.dedup();
                }
            }
            *assigned_items.entry(item.def.id).or_default() -= 1;
            assigned_items.retain(|_, v| *v > 0);
        }
        things
    }

    fn class_is_placeable(&self, class: u32) -> bool {
        self.locs_by_class.get(&class).copied().unwrap_or_default() > 0
    }

    fn spare_space_for_class(&self, class: u32) -> bool {
        self.locs_by_class.get(&class).copied().unwrap_or_default() >= self.items_by_class.get(&class).copied().unwrap_or_default()
    }

    fn find_item_home<R: Rng + ?Sized>(&mut self, item: Item<'a>, rng: &mut R) -> Option<&'a Location> {
        use ItemCategory::*;
        // self.open_locations.shuffle(rng);
        
        for loc in self.open_locations.iter().rev() {
            match (loc.category, item.def.category) {
                (Class(a), Class(b)) if a == b => return Some(loc),
                (_, Class(_)) => (),
                (Class(n), _) if self.spare_space_for_class(n) => return Some(loc),
                (Class(_), _) => (),
                _ => return Some(loc)
            }
        }
        None
        
    }
    fn place_item(&mut self, item: Item<'a>, location: &'a Location, opened: &[&'a Location]) {
        if matches!(location.category, ItemCategory::Class(_)) && !matches!(item.def.category, ItemCategory::Class(_)) {
            println!("!!! Placing a non-dungeon item ({}) in a dungeon slot ({})", item.def.name, location.name);
        }
        self.assignments.insert(location, item);
        self.open_locations.retain(|l| *l != location);
        if !opened.is_empty() {
            self.open_locations.extend(opened.iter().cloned());
            for loc in opened {
                if let ItemCategory::Class(n) = loc.category {
                    *self.locs_by_class.entry(n).or_default() += 1;
                }
            }
        }
        if let ItemCategory::Class(n) = item.def.category {
            if let Some(entry) = self.items_by_class.get_mut(&n) {
                *entry -= 1;
            } else {
                eprintln!("Item {} ({:?}) not in items_by_class?", item.def.name, item.def.category);
            }
        }
        if let ItemCategory::Class(n) = location.category {
            *self.locs_by_class.get_mut(&n).unwrap() -= 1;
        }
        self.closed_locations.retain(|l| !opened.contains(l));
        if let Some((idx, _)) = self.item_pool.iter().enumerate().find(|(_, i)| **i == item) {
            self.item_pool.swap_remove(idx);
        }
        *self.assigned_items.entry(item.def.id).or_default() += item.count;
    }

    fn allocation_round<R: Rng + ?Sized>(&mut self, rng: &mut R) {
        let unlocks = self.single_item_location_unlocks();
        let mut unlock_items: Vec<_> = unlocks.keys().collect();
        // Try to place restricted items first
        // Get item with highest class
        unlock_items.shuffle(rng);
        unlock_items.sort_by_key(|item| std::cmp::Reverse(item.def.category));
        
        for item in unlock_items {
            if let Some(location) = self.find_item_home(*item, rng) {
                println!("Placing {} in {} to unlock new locations", item.def.name, location.name);
                for loc in &unlocks[item] {
                    println!("  - {}", loc.name);
                }
                self.place_item(*item, location, &unlocks[item]);
                return;
            }
        }

        // No single item unlocks a new location; pick one that at least shows up in an unsatisfied goal
        let mut missing_items = HashMap::new();
        for loc in &self.closed_locations {
            loc.requirement.missing(&self.assigned_items, &mut missing_items);
        }
        let (mut restricted_missing_items, mut general_missing_items): (Vec<_>, Vec<_>) = missing_items.into_iter().map(|(id, _)| id).partition(|id| self.logic.get_item(*id).unwrap().category.is_restricted());
        // Nothing missing for locations? Pick a flag instead
        let mut flag_items = HashMap::new();
        for flag in self.logic.flags() {
            flag.requirement.missing(&self.assigned_items, &mut flag_items);
        }
        let (mut restricted_flag_items, mut general_flag_items): (Vec<_>, Vec<_>) = flag_items.into_iter().map(|(id, _)| id).partition(|id| self.logic.get_item(*id).unwrap().category.is_restricted());
        restricted_missing_items.shuffle(rng);
        general_missing_items.shuffle(rng);
        restricted_flag_items.shuffle(rng);
        general_flag_items.shuffle(rng);
        
        for to_add in restricted_missing_items.into_iter().chain(restricted_flag_items).chain(general_missing_items).chain(general_flag_items) {
            // Find a matching item in the pool
            if let Some(&item) = self.item_pool.iter().find(|i| i.def.id == to_add) {
                if let Some(location) = self.find_item_home(item, rng) {
                    println!("Placing {} in {}, hoping that it frees things up", item.def.name, location.name);
                    self.place_item(item, location, &[]);
                    return;
                } else {
                    if let ItemCategory::Class(n) = item.def.category {
                        if !self.class_is_placeable(n) {
                            continue;
                        }
                    }
                    // if let ItemCategory::Class(n) = item.def.category {
                    //     println!("Open locations with matching class: ");
                    //     for loc in &self.open_locations {
                    //         if loc.category == ItemCategory::Class(n) {
                    //             println!("  * {}", loc.name);
                    //         }
                    //     }
                    //     println!("Closed locations with matching class: ");
                    //     for loc in &self.closed_locations {
                    //         if loc.category == ItemCategory::Class(n) {
                    //             println!("  * {}", loc.name);
                    //         }
                    //     }
                    // }
                    println!("!! Wanted to place {} but couldn't find space for it!", item.def.name);

                }
            } else {
                eprintln!("Requirement on an item not found in the item pool: {:?}", self.logic.get_item(to_add).unwrap().name);
            }
        }

        // No critical items left; pick one at random
        // At this point there *should* only be minor items left
        if let Some(&item) = self.item_pool.get(0) {
            if let Some(location) = self.find_item_home(item, rng) {
                println!("Placing {} in {}, to fill up space", item.def.name, location.name);
                self.place_item(item, location, &[]);
            } else {
                println!("Unable to find home for item {}; location pool size: {}", item.def.name, self.open_locations.len());
                self.item_pool.shuffle(rng);
                if let ItemCategory::Class(n) = item.def.category {
                    println!("Open locations with matching class: ");
                    for loc in &self.open_locations {
                        if loc.category == ItemCategory::Class(n) {
                            println!("  * {}", loc.name);
                        }
                    }
                    println!("Closed locations with matching class: ");
                    for loc in &self.closed_locations {
                        if loc.category == ItemCategory::Class(n) {
                            println!("  * {}", loc.name);
                        }
                    }
                }
            }
        }
    }

    pub fn allocate<R: Rng + ?Sized>(&mut self, rng: &mut R) {
        println!("Open locations: ");
        for loc in &self.open_locations {
            println!("  * {}", loc.name);
        }
        println!("Closed locations: ");
        for loc in &self.closed_locations {
            println!("  * {}", loc.name);
        }
        let mut n = 0;
        while !self.item_pool.is_empty() {
            self.allocation_round(rng);
            println!("Open: {}; Closed: {}", self.open_locations.len(), self.closed_locations.len());
            n += 1;
            if n > 300 {
                eprintln!("Not making progress; giving up");
                break;
            }
        }
        println!("\n\nAssignments: ");
        for (loc, item) in &self.assignments {
            println!("  {} -> {}", loc.name, item.def.name);
        }

        self.check_assignments(&self.assignments);
    }

    fn check_assignments(&self, assignments: &HashMap<&Location, Item>) {
        let mut graph = "digraph G {\n".to_string();
        let mut open_locations = HashSet::new();
        let mut acquired_items = HashMap::new();
        let mut new_locations: Vec<_> = self.logic.locations().filter(|l| l.requirement.satisfied(&acquired_items)).collect();
        let mut generations = vec![];
        let mut item_indices: HashMap<ItemId, usize> = HashMap::new();
        let mut completed_flags: HashSet<&Flag> = HashSet::new();
        let mut new_flags: Vec<&Flag> = vec![];
        while !new_locations.is_empty() || !new_flags.is_empty() {
            let mut this_gen = vec![];
            for flag in new_flags {
                completed_flags.insert(flag);
                graph.push_str(&format!(r#"  "flag{}" [label="{}", shape="octagon"];"#, flag.name, flag.name));
                graph.push('\n');
                for sat in flag.requirement.satisfied_by(self.logic, &acquired_items) {
                    let sat = self.logic.get_item(sat).unwrap(); 
                    if sat.category != ItemCategory::Major {
                        continue;
                    }
                    let max_idx = *item_indices.get(&sat.id).unwrap();
                    for i in 1..=max_idx {
                        graph.push_str(&format!(r#"  "flag{}" -> "{}{}";"#, flag.name, sat.name, i));
                        graph.push('\n');
                    }
                }
            }

            for loc in &new_locations {
                if let Some(item) = assignments.get(loc) {
                    if item.def.category != ItemCategory::Major {
                        continue;
                    }
                    let idx = item_indices.entry(item.def.id).or_default();
                    *idx += 1;
                    let idx = *idx;
                    graph.push_str(&format!(r#"  "{}{}" [label="{}\n{}", shape="box"];"#, item.def.name, idx, loc.name, item.def.name));
                    graph.push('\n');
                    for sat in loc.requirement.satisfied_by(self.logic, &acquired_items) {
                        let sat = self.logic.get_item(sat).unwrap(); 
                        if sat.category != ItemCategory::Major {
                            continue;
                        }
                        let max_idx = *item_indices.get(&sat.id).unwrap();
                        for i in 1..=max_idx {
                            if i == idx && sat.id == item.def.id {
                                // Don't link an item to itself
                                continue;
                            }
                            graph.push_str(&format!(r#"  "{}{}" -> "{}{}";"#, item.def.name, idx, sat.name, i));
                            graph.push('\n');
                        }
                    }
                    
                } else {
                    // graph.push_str(&format!(r#"  "{}" [label="{}\n{}", shape="box"];"#, item.def.name, loc.name, item.def.name));
                }
            }
            for loc in new_locations {
                open_locations.insert(loc);
                if let Some(item) = assignments.get(loc) {
                    *acquired_items.entry(item.def.id).or_default() += item.count;
                    this_gen.push((*item, loc));
                } else {
                    eprintln!("Empty location?");
                }
            }
            for (item, loc) in &this_gen {
                print!("{} ({}), ", item.def.name, loc.name);
            }
            println!();
            generations.push(this_gen);
            
            new_locations = self.logic.locations()
                .filter(|l| !open_locations.contains(l))
                .filter(|l| l.requirement.satisfied(&acquired_items)).collect();
            new_flags = self.logic.flags()
                .filter(|f| !completed_flags.contains(f))
                .filter(|f| f.requirement.satisfied(&acquired_items)).collect();
        }
        graph.push('}');
        std::fs::write("graph.dot", graph.as_bytes());
        for flag in self.logic.flags().filter(|f| !completed_flags.contains(f)) {
            println!("Unsatisfied flag: {}", flag.name);
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

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum ItemCategory {
    Minor,
    Major,    
    Class(u32)
}
impl ItemCategory {
    pub fn is_restricted(self) -> bool {
        matches!(self, ItemCategory::Class(_))
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ItemDef {
    name: String,
    id: ItemId,
    category: ItemCategory
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Item<'a> {
    pub def: &'a ItemDef,
    pub count: usize
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
    requirement: ItemCondition
}