pub use super::condition::*;
use super::*;
use rand::prelude::*;
use std::collections::{HashMap, HashSet};

pub struct Allocator {
    item_pool: Conditionals<ItemDef>,
    open_locations: Vec<Rc<Location>>,
    closed_locations: Conditionals<Location>,
    locations: Conditionals<Location>,
    flags: Conditionals<Flag>,
    prefer_new_locations: bool,
    match_category: bool,
    temperature: u32,
    assigned_items: HashMap<Rc<ItemDef>, usize>,

    assignments: HashMap<Rc<Location>, Rc<ItemDef>>,
}

impl Allocator {
    pub fn new(
        item_pool: Conditionals<ItemDef>,
        locations: Conditionals<Location>,
        flags: Conditionals<Flag>,
        prefer_new_locations: bool,
        match_category: bool,
        temperature: u32,
    ) -> Self {
        let mut me = Allocator {
            item_pool,
            prefer_new_locations,
            match_category,
            temperature,
            locations,
            flags,
            open_locations: Default::default(),
            closed_locations: Default::default(),
            assigned_items: Default::default(),
            assignments: Default::default(),
        };
        me.find_open_locs();
        // me.preflight_check();
        me
    }

    fn preflight_check(&self) {
        for (_, req) in &self.locations {
            let mut req = req.clone();
            for (item, _) in &self.item_pool {
                req.assume_item(item, 1)
            }
            assert!(req == ItemCondition::NoRequirements);
        }
        for (_, req) in &self.flags {
            let mut req = req.clone();
            for (item, _) in &self.item_pool {
                req.assume_item(item, 1)
            }
            assert!(req == ItemCondition::NoRequirements);
        }
        for (_, req) in &self.item_pool {
            let mut req = req.clone();
            for (item, _) in &self.item_pool {
                req.assume_item(item, 1)
            }
            assert!(req == ItemCondition::NoRequirements);
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
                        .filter_map(|(i, _)| if i.category == cat { Some(1) } else { None })
                        .count(),
                    self.locations
                        .iter()
                        .filter(|(l, _)| l.category == cat)
                        .count(),
                    "Category count mismatch: {:?}",
                    cat
                );
            }
        } else {
            assert_eq!(self.item_pool.len(), self.locations.len());
        }
        let mut restrictions: Vec<_> = self
            .item_pool
            .iter()
            .filter_map(|(item, _)| item.restriction)
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
                                |(i, _)| i.category == cat && i.restriction == Some(restriction)
                            )
                            .count()
                            <= self
                                .locations
                                .iter()
                                .filter(|(l, _)| l.category == cat && l.restriction == Some(restriction))
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
                        .filter(|(i, _)| i.restriction == Some(restriction))
                        .count()
                        <= self
                            .locations
                            .iter()
                            .filter(|(l, _)| l.restriction == Some(restriction))
                            .count(),
                    "Not enough homes for locations with restriction {}",
                    restriction
                );
            }
        }
    }

    fn find_open_locs(&mut self) {
        for (location, req) in self.locations.iter() {
            if req.satisfied() {
                if !self.assignments.contains_key(location) {
                    self.open_locations.push(location.clone());
                }
            } else {
                self.closed_locations.push((location.clone(), req.clone()));
            }
        }
    }

    fn single_item_location_unlocks(&self) -> HashMap<Rc<ItemDef>, Vec<Rc<Location>>> {
        let mut things: HashMap<Rc<ItemDef>, Vec<_>> = HashMap::new();
        for item in self.placeable_items() {
            for (loc, req) in &self.closed_locations {
                if req.would_be_satisfied_by(&item) {
                    let entry = things.entry(item.clone()).or_default();
                    entry.push(loc.clone());
                    entry.sort_unstable();
                    entry.dedup();
                }
            }
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
                .filter(|(item, _)| item.category == cat && item.restriction == Some(restriction))
                .count()
    }

    fn can_place_in(&self, item: &ItemDef, loc: &Location) -> bool {
        if self.match_category && loc.category != item.category {
            false
        } else {
            match (loc.restriction, item.restriction) {
                (Some(a), Some(b)) if a == b => true,
                (_, Some(_)) => false,
                (Some(n), _) if self.spare_space_for_restriction(loc.category, n) => true,
                (Some(_), _) => false,
                _ => true,
            }
        }
    }

    fn find_item_home<R: Rng + ?Sized>(&self, item: &ItemDef, rng: &mut R) -> Option<Rc<Location>> {
        let mut locations = self.open_locations.clone();
        if !self.prefer_new_locations || item.category == ItemCategory::Minor {
            locations.shuffle(rng);
        }

        for loc in locations.iter().rev() {
            if self.can_place_in(item, &*loc) {
                return Some(loc.clone());
            }
        }
        None
    }
    fn place_item(&mut self, item: &Rc<ItemDef>, location: &Rc<Location>) {
        if location.restriction.is_some() && item.restriction.is_none() {
            println!(
                "!!! Placing a non-dungeon item ({}) in a dungeon slot ({})",
                item, location
            );
        }
        self.assignments.insert(location.clone(), item.clone());
        self.open_locations.retain(|l| l != location);
        let mut opened = vec![];
        for (loc, req) in &mut self.closed_locations {
            let old_req = req.clone();
            req.assume_item(item, 1);
            *req = req.simplify();
            if req.satisfied() {
                opened.push(loc.clone());
                println!("  Unlocked location {}: {}", loc, old_req);
            }
        }
        if !opened.is_empty() {
            self.open_locations.extend(opened);
        }

        self.closed_locations.retain(|(_, req)| !req.satisfied());
        if let Some((idx, _)) = self
            .item_pool
            .iter()
            .enumerate()
            .find(|(_, (i, _))| i == item)
        {
            self.item_pool.swap_remove(idx);
        }
        *self.assigned_items.entry(item.clone()).or_default() += 1;

        for (_, req) in &mut self.item_pool {
            req.assume_item(item, 1);
            *req = req.simplify();
        }
        for (flag, req) in &mut self.flags {
            let old_req = req.clone();
            req.assume_item(item, 1);
            *req = req.simplify();
            if *req == ItemCondition::NoRequirements && *req != old_req {
                println!("  Unlocked flag {}: {}", flag.name, old_req);
            }
        }
        for (_, req) in &mut self.locations {
            req.assume_item(item, 1);
            *req = req.simplify();
        }
    }

    fn probably_safe_to_backfill(
        &self,
        progress_items: &HashSet<Rc<ItemDef>>,
        cat: ItemCategory,
    ) -> bool {
        if self.match_category {
            progress_items
                .iter()
                .filter(|item| item.category == cat && item.restriction.is_none())
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
                .filter(|item| item.restriction.is_none())
                .count()
                * 2
                < self
                    .open_locations
                    .iter()
                    .filter(|l| l.restriction.is_none())
                    .count()
        }
    }

    fn progression_affecting_items(&self) -> HashMap<Rc<ItemDef>, usize> {
        let mut rv = HashMap::new();
        for (_, req) in &self.closed_locations {
            req.missing(&mut rv)
        }
        rv.into_iter().filter(|&(_, count)| count == 1).collect()
    }

    fn placeable_items(&self) -> impl Iterator<Item = Rc<ItemDef>> + '_ {
        let open_restrictions: HashSet<_> = self
            .open_locations
            .iter()
            .filter_map(|loc| loc.restriction)
            .collect();
        self.item_pool
            .iter()
            .filter(move |(item, req)| {
                item.restriction
                    .map(|res| open_restrictions.contains(&res))
                    .unwrap_or(true)
                    && req.satisfied()
            })
            .map(|(item, _)| item.clone())
    }

    fn backfill<R: Rng + ?Sized>(&mut self, rng: &mut R) {
        let progress_items: HashSet<_> =
            self.progression_affecting_items().keys().cloned().collect();
        for cat in [
            ItemCategory::Minor,
            ItemCategory::Major,
            ItemCategory::DungeonItem,
        ] {
            if self.probably_safe_to_backfill(&progress_items, cat) {
                let mut placeable: Vec<_> = self
                    .placeable_items()
                    .filter(|item| {
                        item.category == cat
                            && !progress_items.contains(item)
                            && item.restriction.is_none()
                    })
                    .map(|item| {
                        let weight = item.weight + rng.gen_range(0..self.temperature);
                        (item, weight)
                    })
                    .collect();
                placeable.sort_unstable_by_key(|(_, weight)| *weight);
                if placeable.is_empty() {
                    continue;
                }

                // // Only place half (rounding down) available items on any given pass
                // placeable.truncate(
                //     self.open_locations
                //         .iter()
                //         .filter(|l| l.category == cat)
                //         .count()
                //         / 2,
                // );
                // We pop from the end, so reverse the list
                placeable.reverse();

                while self.probably_safe_to_backfill(&progress_items, cat) {
                    if let Some((item, weight)) = placeable.pop() {
                        // Locations are sorted by when they were opened, so we'll try to fill earlier locations first
                        if let Some(location) = self.open_locations.iter().find(|&loc| {
                            self.can_place_in(&*item, &**loc) && loc.restriction.is_none()
                        }) {
                            let location = location.clone();
                            println!("Backfilling {} ({}) in {}", item, weight, location);
                            self.place_item(&item, &location);
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
        // Check that closed locations are still actually closed
        for (loc, req) in &self.closed_locations {
            if req.satisfied() {
                println!("!!! Closed location {} should be open! {}", loc, req);
            }
        }

        self.backfill(rng);
        // And again
        for (loc, req) in &self.closed_locations {
            if req.satisfied() {
                println!("!!!2 Closed location {} should be open! {}", loc, req);
            }
        }
        let unlocks = self.single_item_location_unlocks();
        let mut unlock_items: Vec<_> = unlocks
            .iter()
            .map(|(item, _)| (item, item.weight + rng.gen_range(0..self.temperature)))
            .collect();
        unlock_items.sort_unstable_by_key(|(_, weight)| *weight);

        print!("Capabilities: ");
        for (flag, req) in self.flags.iter() {
            if req.satisfied() {
                print!("{} ", flag.name);
            }
        }
        println!();

        // println!("{:?}", unlock_items);

        for (item, weight) in unlock_items {
            if let Some(location) = self.find_item_home(&*item, rng) {
                println!(
                    "Placing {} ({}) in {} to unlock new locations",
                    item, weight, location
                );
                // for loc in &unlocks[item] {
                //     println!(" - {}", loc);
                // }
                self.place_item(item, &location);
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
        for (_, req) in &self.closed_locations {
            req.missing(&mut missing_items);
        }
        let (mut restricted_missing_items, mut general_missing_items): (Vec<_>, Vec<_>) =
            missing_items
                .into_iter()
                .map(|(item, _)| item)
                .partition(|item| item.restriction.is_some());
        // Nothing missing for locations? Pick a flag instead
        let mut flag_items = HashMap::new();
        for (_flag, req) in self.flags.iter() {
            req.missing(&mut flag_items);
        }
        let (mut restricted_flag_items, mut general_flag_items): (Vec<_>, Vec<_>) = flag_items
            .into_iter()
            .map(|(item, _)| item)
            .partition(|item| item.restriction.is_some());
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
            let possible_match = self.placeable_items().find(|i| i == &to_add);
            if let Some(item) = possible_match {
                if let Some(location) = self.find_item_home(&*item, rng) {
                    println!(
                        "Placing {} in {}, hoping that it frees things up",
                        item, location
                    );
                    self.place_item(&item, &location);
                    return;
                } else {
                    if let Some(n) = item.restriction {
                        if !self.restricted_item_is_placeable(item.category, n) {
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
            }
        }

        // No critical items left; pick one at random
        // At this point there *should* only be minor items left
        let placeable_items: Vec<_> = self.placeable_items().collect();
        for item in placeable_items {
            if let Some(location) = self.find_item_home(&*item, rng) {
                println!("Placing {} in {}, to fill up space", item, location);
                self.place_item(&item, &location);
                return;
            } else {
                println!(
                    "Unable to find home for item {}; location pool size: {}",
                    item,
                    self.open_locations.len()
                );
                self.item_pool.shuffle(rng);
                if let Some(n) = item.restriction {
                    println!("Open locations with matching class: ");
                    for loc in &self.open_locations {
                        if loc.restriction == Some(n) {
                            println!("  * {}", loc);
                        }
                    }
                    println!("Closed locations with matching class: ");
                    for (loc, req) in &self.closed_locations {
                        if loc.restriction == Some(n) {
                            println!("  * {}: {}", loc, req);
                        }
                    }
                }
            }
        }
        println!("Open locations: ");
        for loc in &self.open_locations {
            println!(" - {}", loc);
        }
        println!("Unassigned items:");
        for (item, _) in &self.item_pool {
            println!(" - {}", item);
        }
    }

    fn alloc_progress(&self) -> Result<String, std::fmt::Error> {
        use std::fmt::Write;
        let mut restrictions: Vec<_> = self
            .locations
            .iter()
            .filter_map(|(l, _)| l.restriction)
            .collect();
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
                .filter(|(l, _)| l.category == ItemCategory::Major)
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
                        .filter(|(l, _)| l.restriction == Some(restriction))
                        .count(),
                    self.closed_locations
                        .iter()
                        .filter(|(l, _)| l.category == ItemCategory::Major
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
                .filter(|(i, _)| i.category == ItemCategory::Major)
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
                        .filter(|(i, _)| i.restriction == Some(restriction))
                        .count(),
                    self.item_pool
                        .iter()
                        .filter(|(i, _)| i.category == ItemCategory::Major
                            && i.restriction == Some(restriction))
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
                .filter_map(|(item, count)| {
                    if item.category == ItemCategory::Major {
                        Some(count)
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
                        .filter_map(|(item, count)| {
                            if item.restriction == Some(restriction) {
                                Some(count)
                            } else {
                                None
                            }
                        })
                        .sum::<usize>(),
                    self.assigned_items
                        .iter()
                        .filter_map(|(item, count)| {
                            if item.category == ItemCategory::Major
                                && item.restriction == Some(restriction)
                            {
                                Some(count)
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

    pub fn allocate<R: Rng + ?Sized>(&mut self, rng: &mut R) -> HashMap<Rc<Location>, Rc<ItemDef>> {
        println!("Open locations: ");
        for loc in &self.open_locations {
            println!("  * {}", loc);
        }
        println!("Closed locations: ");
        for (loc, req) in &self.closed_locations {
            println!("  * {} {}", loc, req);
        }
        let mut n = 0;
        self.item_pool.shuffle(rng);
        while !self.item_pool.is_empty() {
            self.allocation_round(rng);
            println!("{}", self.alloc_progress().unwrap());

            n += 1;
            if n > 150 {
                eprintln!("Not making progress; giving up");
                println!("Closed locations:");
                for (loc, req) in &self.closed_locations {
                    println!("  - {}", loc);
                    print!("    ");
                    let mut req = req.clone();
                    for (id, count) in &self.assigned_items {
                        req.assume_item(id, *count);
                    }
                    println!("{}", req);
                }
                break;
            }
        }
        println!("\n\nAssignments: ");
        for (loc, item) in &self.assignments {
            println!("  {} -> {}", loc, item);
        }

        self.assignments.clone()
    }
}
