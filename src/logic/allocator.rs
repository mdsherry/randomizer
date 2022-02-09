pub use super::condition::*;
use super::*;
use rand::prelude::*;
use std::collections::{HashMap, HashSet};

pub struct Allocator<'a> {
    logic: &'a Logic,
    item_pool: Vec<Item<'a>>,
    open_locations: Vec<&'a Location>,
    closed_locations: Vec<&'a Location>,
    prefer_new_locations: bool,
    match_category: bool,
    temperature: u32,
    assigned_items: HashMap<ItemId, usize>,
    items_by_restriction: HashMap<Restriction, usize>,
    locs_by_restriction: HashMap<Restriction, usize>,

    assignments: HashMap<&'a Location, Item<'a>>,
}

impl<'a> Allocator<'a> {
    pub fn new(
        logic: &'a Logic,
        item_pool: Vec<Item<'a>>,
        prefer_new_locations: bool,
        match_category: bool,
        temperature: u32,
    ) -> Self {
        let mut me = Allocator {
            logic,
            item_pool,
            prefer_new_locations,
            match_category,
            temperature,
            open_locations: Default::default(),
            closed_locations: Default::default(),
            assigned_items: Default::default(),
            items_by_restriction: Default::default(),
            locs_by_restriction: Default::default(),
            assignments: Default::default(),
        };
        me.find_open_locs();
        me.compute_locs_by_class();
        me.preflight_check();
        me
    }

    fn preflight_check(&self) {
        let mut all_items = HashMap::new();

        for item in &self.item_pool {
            *all_items.entry(item.def.id).or_default() += item.count;
        }
        for location in self.logic.locations() {
            assert!(location.requirement.satisfied(&all_items));
        }
        if self.match_category {
            for cat in [
                ItemCategory::Minor,
                ItemCategory::Major,
                ItemCategory::DungeonItem,
            ] {
                assert_eq!(
                    self.item_pool
                        .iter()
                        .filter_map(|i| if i.def.category == cat {
                            Some(i.count)
                        } else {
                            None
                        })
                        .count(),
                    self.logic.locations().filter(|l| l.category == cat).count(),
                    "Category count mismatch: {:?}",
                    cat
                );
            }
        } else {
            assert_eq!(self.item_pool.len(), self.logic.locations().count());
        }
        let mut restrictions: Vec<_> = self
            .item_pool
            .iter()
            .filter_map(|item| item.def.restriction)
            .collect();
        restrictions.sort_unstable();
        for restriction in restrictions {
            if self.match_category {
                for cat in [
                    ItemCategory::Minor,
                    ItemCategory::Major,
                    ItemCategory::DungeonItem,
                ] {
                    assert!(
                        self.item_pool
                            .iter()
                            .filter(
                                |i| i.def.category == cat && i.def.restriction == Some(restriction)
                            )
                            .count()
                            <= self
                                .logic
                                .locations()
                                .filter(|l| l.category == cat && l.restriction == Some(restriction))
                                .count(),
                        "Not enough homes for locations with restriction {} and category {:?}",
                        restriction,
                        cat
                    );
                }
            } else {
                assert!(
                    self.item_pool
                        .iter()
                        .filter(|i| i.def.restriction == Some(restriction))
                        .count()
                        <= self
                            .logic
                            .locations()
                            .filter(|l| l.restriction == Some(restriction))
                            .count(),
                    "Not enough homes for locations with restriction {}",
                    restriction
                );
            }
        }
    }
    fn compute_locs_by_class(&mut self) {
        for item in &self.item_pool {
            if let Some(n) = item.def.restriction {
                *self.items_by_restriction.entry(n).or_insert(0) += 1;
            }
        }
        for loc in &self.open_locations {
            if let Some(n) = loc.restriction {
                *self.locs_by_restriction.entry(n).or_insert(0) += 1;
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

    fn restricted_item_is_placeable(&self, cat: ItemCategory, restriction: Restriction) -> bool {
        self.open_locations
            .iter()
            .filter(|l| l.category == cat && l.restriction == Some(restriction))
            .count()
            > 0
    }

    fn spare_space_for_restriction(&self, cat: ItemCategory, restriction: Restriction) -> bool {
        self.open_locations
            .iter()
            .filter(|l| l.category == cat && l.restriction == Some(restriction))
            .count()
            > self
                .item_pool
                .iter()
                .filter(|item| {
                    item.def.category == cat && item.def.restriction == Some(restriction)
                })
                .count()
    }

    fn can_place_in(&self, item: Item, loc: &Location) -> bool {
        if self.match_category && loc.category != item.def.category {
            false
        } else {
            match (loc.restriction, item.def.restriction) {
                (Some(a), Some(b)) if a == b => true,
                (_, Some(_)) => false,
                (Some(n), _) if self.spare_space_for_restriction(loc.category, n) => true,
                (Some(_), _) => false,
                _ => true,
            }
        }
    }

    fn find_item_home<R: Rng + ?Sized>(
        &mut self,
        item: Item<'a>,
        rng: &mut R,
    ) -> Option<&'a Location> {
        let mut locations = self.open_locations.clone();
        if !self.prefer_new_locations || item.def.category == ItemCategory::Minor {
            locations.shuffle(rng);
        }

        for loc in locations.iter().rev() {
            if self.can_place_in(item, *loc) {
                return Some(loc);
            }
        }
        None
    }
    fn place_item(&mut self, item: Item<'a>, location: &'a Location, opened: &[&'a Location]) {
        if location.restriction.is_some() && item.def.restriction.is_none() {
            println!(
                "!!! Placing a non-dungeon item ({}) in a dungeon slot ({})",
                item, location
            );
        }
        self.assignments.insert(location, item);
        self.open_locations.retain(|l| *l != location);
        if !opened.is_empty() {
            self.open_locations.extend(opened.iter().cloned());
            for loc in opened {
                if let Some(n) = loc.restriction {
                    *self.locs_by_restriction.entry(n).or_default() += 1;
                }
            }
        }
        if let Some(n) = item.def.restriction {
            if let Some(entry) = self.items_by_restriction.get_mut(&n) {
                println!(
                    "Reducing restricted item count by 1 for {:?} from {}",
                    n, entry
                );
                *entry -= 1;
            } else {
                eprintln!(
                    "Item {} ({:?}) not in items_by_class?",
                    item, item.def.category
                );
            }
        }
        if let Some(n) = location.restriction {
            *self.locs_by_restriction.get_mut(&n).unwrap() -= 1;
        }
        self.closed_locations.retain(|l| !opened.contains(l));
        if let Some((idx, _)) = self.item_pool.iter().enumerate().find(|(_, i)| **i == item) {
            self.item_pool.swap_remove(idx);
        }
        *self.assigned_items.entry(item.def.id).or_default() += item.count;
    }

    fn probably_safe_to_backfill(
        &self,
        progress_items: &HashSet<ItemId>,
        cat: ItemCategory,
    ) -> bool {
        if self.match_category {
            progress_items
                .iter()
                .filter(|i| {
                    let item = self.logic.get_item(**i).unwrap();
                    item.category == cat && item.restriction.is_none()
                })
                .count()
                * 2
                < self
                    .open_locations
                    .iter()
                    .filter(|loc| loc.category == cat && loc.restriction.is_none())
                    .count()
        } else {
            progress_items
                .iter()
                .filter(|i| {
                    let item = self.logic.get_item(**i).unwrap();
                    item.restriction.is_none()
                })
                .count()
                * 2
                < self
                    .open_locations
                    .iter()
                    .filter(|l| l.restriction.is_none())
                    .count()
        }
    }

    fn progression_affecting_items(&self) -> HashMap<ItemId, usize> {
        let mut rv = HashMap::new();
        for loc in &self.closed_locations {
            loc.requirement.missing(&self.assigned_items, &mut rv)
        }
        rv.into_iter().filter(|&(_, count)| count == 1).collect()
    }

    fn backfill<R: Rng + ?Sized>(&mut self, rng: &mut R) {
        let progress_items: HashSet<_> =
            self.progression_affecting_items().keys().copied().collect();
        for cat in [
            ItemCategory::Minor,
            ItemCategory::Major,
            ItemCategory::DungeonItem,
        ] {
            if self.probably_safe_to_backfill(&progress_items, cat) {
                let mut placeable: Vec<_> = self
                    .item_pool
                    .iter()
                    .filter(|item| {
                        item.def.category == cat
                            && !progress_items.contains(&item.def.id)
                            && item.def.restriction.is_none()
                    })
                    .copied()
                    .map(|item| (item, item.def.weight + rng.gen_range(0..self.temperature)))
                    .collect();
                placeable.sort_unstable_by_key(|(_, weight)| *weight);
                if placeable.is_empty() {
                    continue;
                }

                // Only place half (rounding down) available items on any given pass
                placeable.truncate(
                    self.open_locations
                        .iter()
                        .filter(|l| l.category == cat)
                        .count()
                        / 2,
                );
                // We pop from the end, so reverse the list
                placeable.reverse();

                while self.probably_safe_to_backfill(&progress_items, cat) {
                    if let Some((item, weight)) = placeable.pop() {
                        // Locations are sorted by when they were opened, so we'll try to fill earlier locations first
                        if let Some(&location) = self
                            .open_locations
                            .iter()
                            .find(|&&loc| self.can_place_in(item, loc) && loc.restriction.is_none())
                        {
                            println!("Backfilling {} ({}) in {}", item, weight, location);
                            self.place_item(item, location, &[]);
                        } else {
                            println!(
                                "(Wanted to backfill {} ({}) but couldn't find a home for it)",
                                item, weight
                            );
                        }
                    } else {
                        break;
                    }
                }
            }
        }
    }
    fn allocation_round<R: Rng + ?Sized>(&mut self, rng: &mut R) {
        self.backfill(rng);
        let unlocks = self.single_item_location_unlocks();
        let mut unlock_items: Vec<_> = unlocks
            .iter()
            .map(|(item, locs)| {
                (
                    item,
                    item.def.weight + rng.gen_range(0..self.temperature),
                    locs.iter().any(|l| l.category == ItemCategory::Major && item.def.restriction == l.restriction),
                    locs.iter().any(|l| l.category == ItemCategory::Major),
                )
            })
            .collect();
        unlock_items.sort_unstable_by_key(|(_, weight, unlocks_restricted_major, unlocks_major)| (!*unlocks_restricted_major, !*unlocks_major, *weight));

        print!("Capabilities: ");
        for flag in self.logic.flags() {
            if flag.requirement.satisfied(&self.assigned_items) {
                print!("{} ", flag.name);
            }
        }
        println!();
        println!("{:?}", unlock_items);

        for (item, weight, _, _) in unlock_items {
            if let Some(location) = self.find_item_home(*item, rng) {
                println!(
                    "Placing {} ({}) in {} to unlock new locations",
                    item, weight, location
                );
                for loc in &unlocks[item] {
                    println!("  - {}", loc);
                }
                self.place_item(*item, location, &unlocks[item]);
                return;
            } else {
                println!(
                    "(Wanted to place {} ({}) but couldn't find a home for it)",
                    item, weight
                );
            }
        }

        // No single item unlocks a new location; pick one that at least shows up in an unsatisfied goal
        let mut missing_items = HashMap::new();
        for loc in &self.closed_locations {
            loc.requirement
                .missing(&self.assigned_items, &mut missing_items);
        }
        let (mut restricted_missing_items, mut general_missing_items): (Vec<_>, Vec<_>) =
            missing_items
                .into_iter()
                .map(|(id, _)| id)
                .partition(|id| self.logic.get_item(*id).unwrap().restriction.is_some());
        // Nothing missing for locations? Pick a flag instead
        let mut flag_items = HashMap::new();
        for flag in self.logic.flags() {
            flag.requirement
                .missing(&self.assigned_items, &mut flag_items);
        }
        let (mut restricted_flag_items, mut general_flag_items): (Vec<_>, Vec<_>) = flag_items
            .into_iter()
            .map(|(id, _)| id)
            .partition(|id| self.logic.get_item(*id).unwrap().restriction.is_some());
        restricted_missing_items.shuffle(rng);
        general_missing_items.shuffle(rng);
        restricted_flag_items.shuffle(rng);
        general_flag_items.shuffle(rng);

        for to_add in restricted_missing_items
            .into_iter()
            .chain(restricted_flag_items)
            .chain(general_missing_items)
            .chain(general_flag_items)
        {
            // Find a matching item in the pool
            if let Some(&item) = self.item_pool.iter().find(|i| i.def.id == to_add) {
                if let Some(location) = self.find_item_home(item, rng) {
                    println!(
                        "Placing {} in {}, hoping that it frees things up",
                        item, location
                    );
                    self.place_item(item, location, &[]);
                    return;
                } else {
                    if let Some(n) = item.def.restriction {
                        if !self.restricted_item_is_placeable(item.def.category, n) {
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
                    println!(
                        "!! Wanted to place {} but couldn't find space for it!",
                        item
                    );
                }
            } else {
                eprintln!(
                    "Requirement on an item not found in the item pool: {:?}",
                    self.logic.get_item(to_add).unwrap().name
                );
            }
        }

        // No critical items left; pick one at random
        // At this point there *should* only be minor items left
        if let Some(&item) = self.item_pool.get(0) {
            if let Some(location) = self.find_item_home(item, rng) {
                println!("Placing {} in {}, to fill up space", item, location);
                self.place_item(item, location, &[]);
                return;
            } else {
                println!(
                    "Unable to find home for item {}; location pool size: {}",
                    item,
                    self.open_locations.len()
                );
                self.item_pool.shuffle(rng);
                if let Some(n) = item.def.restriction {
                    println!("Open locations with matching class: ");
                    for loc in &self.open_locations {
                        if loc.restriction == Some(n) {
                            println!("  * {}", loc);
                        }
                    }
                    println!("Closed locations with matching class: ");
                    for loc in &self.closed_locations {
                        if loc.restriction == Some(n) {
                            println!("  * {}", loc);
                        }
                    }
                }
            }
        }
        println!("Open locations: ");
        for loc in &self.open_locations {
            println!(" - {}", loc);
            loc.requirement.render(self.logic);
        }
        println!("Unassigned items:");
        for item in &self.item_pool {
            println!(" - {}", item);
        }
    }

    fn alloc_progress(&self) -> Result<String, std::fmt::Error> {
        use std::fmt::Write;
        let mut restrictions: Vec<_> = self.logic.items().filter_map(|i| i.restriction).collect();
        restrictions.sort_unstable();
        restrictions.dedup();
        let mut s = "Open: ".to_string();
        write!(
            &mut s,
            "Open: {} ({})",
            self.open_locations.len(),
            self.open_locations
                .iter()
                .filter(|l| l.category == ItemCategory::Major)
                .count()
        )?;
        if !restrictions.is_empty() {
            write!(&mut s, " [")?;
            let mut first = true;
            for &restriction in &restrictions {
                if !first {
                    write!(&mut s, ", ")?;
                }
                first = false;
                write!(
                    &mut s,
                    "{} ({})",
                    self.open_locations
                        .iter()
                        .filter(|l| l.restriction == Some(restriction))
                        .count(),
                    self.open_locations
                        .iter()
                        .filter(|l| l.category == ItemCategory::Major
                            && l.restriction == Some(restriction))
                        .count()
                )?;
            }
            write!(&mut s, "]")?;
        }
        write!(&mut s, "; ")?;
        write!(
            &mut s,
            "Closed: {} ({})",
            self.closed_locations.len(),
            self.closed_locations
                .iter()
                .filter(|l| l.category == ItemCategory::Major)
                .count()
        )?;
        if !restrictions.is_empty() {
            write!(&mut s, " [")?;
            let mut first = true;
            for &restriction in &restrictions {
                if !first {
                    write!(&mut s, ", ")?;
                }
                first = false;
                write!(
                    &mut s,
                    "{} ({})",
                    self.closed_locations
                        .iter()
                        .filter(|l| l.restriction == Some(restriction))
                        .count(),
                    self.closed_locations
                        .iter()
                        .filter(|l| l.category == ItemCategory::Major
                            && l.restriction == Some(restriction))
                        .count()
                )?;
            }
            write!(&mut s, "]")?;
        }
        write!(
            &mut s,
            "; Unassigned items {} ({})",
            self.item_pool.len(),
            self.item_pool
                .iter()
                .filter(|i| i.def.category == ItemCategory::Major)
                .count(),
        )?;
        if !restrictions.is_empty() {
            write!(&mut s, " [")?;
            let mut first = true;
            for &restriction in &restrictions {
                if !first {
                    write!(&mut s, ", ")?;
                }
                first = false;
                write!(
                    &mut s,
                    "{} ({})",
                    self.item_pool
                        .iter()
                        .filter(|i| i.def.restriction == Some(restriction))
                        .count(),
                    self.item_pool
                        .iter()
                        .filter(|i| i.def.category == ItemCategory::Major
                            && i.def.restriction == Some(restriction))
                        .count()
                )?;
            }
            write!(&mut s, "]")?;
        }
        write!(
            &mut s,
            "; Assigned items {} ({})",
            self.assigned_items.values().sum::<usize>(),
            self.assigned_items
                .iter()
                .filter_map(|i| {
                    let item = self.logic.get_item(*i.0).unwrap();
                    if item.category == ItemCategory::Major {
                        Some(i.1)
                    } else {
                        None
                    }
                })
                .sum::<usize>()
        )?;
        if !restrictions.is_empty() {
            write!(&mut s, " [")?;
            let mut first = true;
            for &restriction in &restrictions {
                if !first {
                    write!(&mut s, ", ")?;
                }
                first = false;
                write!(
                    &mut s,
                    "{} ({})",
                    self.assigned_items
                        .iter()
                        .filter_map(|i| {
                            let item = self.logic.get_item(*i.0).unwrap();
                            if item.restriction == Some(restriction) {
                                Some(i.1)
                            } else {
                                None
                            }
                        })
                        .sum::<usize>(),
                    self.assigned_items
                        .iter()
                        .filter_map(|i| {
                            let item = self.logic.get_item(*i.0).unwrap();
                            if item.category == ItemCategory::Major
                                && item.restriction == Some(restriction)
                            {
                                Some(i.1)
                            } else {
                                None
                            }
                        })
                        .sum::<usize>()
                )?;
            }
            write!(&mut s, "]")?;
        }
        //self.assigned_items.values().sum::<usize>()
        Ok(s)
    }

    pub fn allocate<R: Rng + ?Sized>(&mut self, rng: &mut R) {
        println!("Open locations: ");
        for loc in &self.open_locations {
            println!("  * {}", loc);
        }
        println!("Closed locations: ");
        for loc in &self.closed_locations {
            println!("  * {}", loc);
        }
        let mut n = 0;
        self.item_pool.shuffle(rng);
        while !self.item_pool.is_empty() {
            self.allocation_round(rng);
            println!("{}", self.alloc_progress().unwrap());

            n += 1;
            if n > 300 {
                eprintln!("Not making progress; giving up");
                println!("Closed locations:");
                for loc in &self.closed_locations {
                    println!("  - {}", loc);
                    print!("    ");
                    let mut req = loc.requirement.clone();
                    for (id, count) in &self.assigned_items {
                        req.assume_item(*id, *count);
                    }
                    req.render(self.logic);
                    println!();
                }
                break;
            }
        }
        println!("\n\nAssignments: ");
        for (loc, item) in &self.assignments {
            println!("  {} -> {}", loc, item);
        }

        self.check_assignments(&self.assignments);
    }

    fn check_assignments(&self, assignments: &HashMap<&Location, Item>) {
        let mut graph = "digraph G {\n".to_string();
        let mut open_locations = HashSet::new();
        let mut acquired_items = HashMap::new();
        let mut new_locations: Vec<_> = self
            .logic
            .locations()
            .filter(|l| l.requirement.satisfied(&acquired_items))
            .collect();
        let mut generations = vec![];
        let mut item_indices: HashMap<ItemId, usize> = HashMap::new();
        let mut completed_flags: HashSet<&Flag> = HashSet::new();
        let mut new_flags: Vec<&Flag> = vec![];
        while !new_locations.is_empty() || !new_flags.is_empty() {
            let mut this_gen = vec![];
            for &flag in &new_flags {
                completed_flags.insert(flag);
                graph.push_str(&format!(
                    r#"  "flag{}" [label="{}", shape="octagon"];"#,
                    flag.name, flag.name
                ));
                graph.push('\n');
                for sat in flag.requirement.satisfied_by(self.logic, &acquired_items) {
                    let sat = self.logic.get_item(sat).unwrap();
                    if !sat.show_in_graph {
                        continue;
                    }
                    let max_idx = *item_indices.get(&sat.id).unwrap();
                    for i in 1..=max_idx {
                        graph
                            .push_str(&format!(r#"  "flag{}" -> "{}{}";"#, flag.name, sat.name, i));
                        graph.push('\n');
                    }
                }
            }

            for loc in &new_locations {
                if let Some(item) = assignments.get(loc) {
                    if !item.def.show_in_graph {
                        continue;
                    }
                    let idx = item_indices.entry(item.def.id).or_default();
                    *idx += 1;
                    let idx = *idx;
                    graph.push_str(&format!(
                        r#"  "{}{}" [label="{}\n{}", shape="box"];"#,
                        item.def.name, idx, loc, item.def.name
                    ));
                    graph.push('\n');
                    for sat in loc.requirement.satisfied_by(self.logic, &acquired_items) {
                        let sat = self.logic.get_item(sat).unwrap();
                        if !sat.show_in_graph {
                            continue;
                        }
                        let max_idx = *item_indices.get(&sat.id).unwrap();
                        for i in 1..=max_idx {
                            if i == idx && sat.id == item.def.id {
                                // Don't link an item to itself
                                continue;
                            }
                            graph.push_str(&format!(
                                r#"  "{}{}" -> "{}{}";"#,
                                item.def.name, idx, sat.name, i
                            ));
                            graph.push('\n');
                        }
                    }
                } else {
                    // graph.push_str(&format!(r#"  "{}" [label="{}\n{}", shape="box"];"#, item, loc.name, item));
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
            for flag in new_flags {
                print!("{{{}}}, ", flag.name);
            }
            for (item, loc) in &this_gen {
                print!("{} ({}), ", item, loc);
            }
            println!();
            generations.push(this_gen);

            new_locations = self
                .logic
                .locations()
                .filter(|l| !open_locations.contains(l))
                .filter(|l| l.requirement.satisfied(&acquired_items))
                .collect();
            new_flags = self
                .logic
                .flags()
                .filter(|f| !completed_flags.contains(f))
                .filter(|f| f.requirement.satisfied(&acquired_items))
                .collect();
        }
        graph.push('}');
        std::fs::write("graph.dot", graph.as_bytes());
        for flag in self.logic.flags().filter(|f| !completed_flags.contains(f)) {
            println!("Unsatisfied flag: {}", flag.name);
        }
    }
}
