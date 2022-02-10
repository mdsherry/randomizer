pub use super::condition::*;
use super::*;
use rand::prelude::*;
use std::collections::{HashMap, HashSet};

type Conditional<T> = (Rc<T>, ItemCondition);
type Conditionals<T> = Vec<Conditional<T>>;

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
    items_by_restriction: HashMap<Restriction, usize>,
    locs_by_restriction: HashMap<Restriction, usize>,

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

        for (item, _) in &self.item_pool {
            *all_items.entry(item.clone()).or_default() += 1;
        }
        for (_, req) in &self.locations {
            assert!(req.satisfied(&all_items));
        }
        for (_, req) in &self.flags {
            assert!(req.satisfied(&all_items));
        }
        for (_, req) in &self.item_pool {
            assert!(req.satisfied(&all_items));
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
                        .filter_map(|(i, _)| if i.category == cat {
                            Some(1)
                        } else {
                            None
                        })
                        .count(),
                    self.locations.iter().filter(|(l, _)| l.category == cat).count(),
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
    fn compute_locs_by_class(&mut self) {
        for (item, _) in &self.item_pool {
            if let Some(n) = item.restriction {
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
        for (location, req) in self.locations.iter() {
            if req.satisfied(&self.assigned_items) {
                if !self.assignments.contains_key(location) {
                    self.open_locations.push(location.clone());
                }
            } else {
                self.closed_locations.push((location.clone(), req.clone()));
            }
        }
    }

    fn single_item_location_unlocks(&self) -> HashMap<Rc<ItemDef>, Vec<Rc<Location>>> {
        let mut assigned_items = self.assigned_items.clone();
        let mut things: HashMap<Rc<ItemDef>, Vec<_>> = HashMap::new();
        for item in self.placeable_items() {
            *assigned_items.entry(item.clone()).or_default() += 1;
            for (loc, req) in &self.closed_locations {
                if req.satisfied(&assigned_items) {
                    let entry = things.entry(item.clone()).or_default();
                    entry.push(loc.clone());
                    entry.sort_unstable();
                    entry.dedup();
                }
            }
            *assigned_items.entry(item).or_default() -= 1;
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
                .filter(|(item, _)| {
                    item.category == cat && item.restriction == Some(restriction)
                })
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

    fn find_item_home<R: Rng + ?Sized>(
        &self,
        item: &ItemDef,
        rng: &mut R,
    ) -> Option<Rc<Location>> {
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
    fn place_item(&mut self, item: &Rc<ItemDef>, location: &Rc<Location>, opened: &[Rc<Location>]) {
        if location.restriction.is_some() && item.restriction.is_none() {
            println!(
                "!!! Placing a non-dungeon item ({}) in a dungeon slot ({})",
                item, location
            );
        }
        self.assignments.insert(location.clone(), item.clone());
        self.open_locations.retain(|l| l != location);
        if !opened.is_empty() {
            self.open_locations.extend(opened.iter().cloned());
            for loc in opened {
                if let Some(n) = loc.restriction {
                    *self.locs_by_restriction.entry(n).or_default() += 1;
                }
            }
        }
        if let Some(n) = item.restriction {
            if let Some(entry) = self.items_by_restriction.get_mut(&n) {
                println!(
                    "Reducing restricted item count by 1 for {:?} from {}",
                    n, entry
                );
                *entry -= 1;
            } else {
                eprintln!(
                    "Item {} ({:?}) not in items_by_class?",
                    item, item.category
                );
            }
        }
        if let Some(n) = location.restriction {
            *self.locs_by_restriction.get_mut(&n).unwrap() -= 1;
        }
        self.closed_locations.retain(|(l, _)| !opened.contains(l));
        if let Some((idx, _)) = self.item_pool.iter().enumerate().find(|(_, (i, _))| i == item) {
            self.item_pool.swap_remove(idx);
        }
        *self.assigned_items.entry(item.clone()).or_default() += 1;
    }

    fn probably_safe_to_backfill(
        &self,
        progress_items: &HashSet<Rc<ItemDef>>,
        cat: ItemCategory,
    ) -> bool {
        if self.match_category {
            progress_items
                .iter()
                .filter(|item| {
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
                .filter(|item| {
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

    fn progression_affecting_items(&self) -> HashMap<Rc<ItemDef>, usize> {
        let mut rv = HashMap::new();
        for (_, req) in &self.closed_locations {
            req.missing(&self.assigned_items, &mut rv)
        }
        rv.into_iter().filter(|&(_, count)| count == 1).collect()
    }

    fn placeable_items(&self) -> impl Iterator<Item=Rc<ItemDef>> + '_ {
        self.item_pool
            .iter()
            .filter(|(_, req)| req.satisfied(&self.assigned_items))
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
                        if let Some(location) = self
                            .open_locations
                            .iter()
                            .find(|&loc| self.can_place_in(&*item, &**loc) && loc.restriction.is_none())
                        {
                            let location = location.clone();
                            println!("Backfilling {} ({}) in {}", item, weight, location);
                            self.place_item(&item, &location, &[]);
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
            .map(|(item, _)| {
                (
                    item,
                    item.weight + rng.gen_range(0..self.temperature)
                )
            })
            .collect();
        unlock_items.sort_unstable_by_key(|(_, weight)| *weight);

        print!("Capabilities: ");
        for (flag, req) in self.flags.iter() {
            if req.satisfied(&self.assigned_items) {
                print!("{} ", flag.name);
            }
        }
        println!();
        println!("{:?}", unlock_items);

        for (item, weight) in unlock_items {
            if let Some(location) = self.find_item_home(&*item, rng) {
                println!(
                    "Placing {} ({}) in {} to unlock new locations",
                    item, weight, location
                );
                for loc in &unlocks[item] {
                    println!("  - {}", loc);
                }
                self.place_item(item, &location, &unlocks[item]);
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
            req.missing(&self.assigned_items, &mut missing_items);
        }
        let (mut restricted_missing_items, mut general_missing_items): (Vec<_>, Vec<_>) =
            missing_items
                .into_iter()
                .map(|(item, _)| item)
                .partition(|item| item.restriction.is_some());
        // Nothing missing for locations? Pick a flag instead
        let mut flag_items = HashMap::new();
        for (_flag, req) in self.flags.iter() {
            req
                .missing(&self.assigned_items, &mut flag_items);
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
            let possible_match  = self.placeable_items().find(|i| i == &to_add);
            if let Some(item) = possible_match {
                if let Some(location) = self.find_item_home(&*item, rng) {
                    println!(
                        "Placing {} in {}, hoping that it frees things up",
                        item, location
                    );
                    self.place_item(&item, &location, &[]);
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
            } else {
                eprintln!(
                    "Requirement on an item not found in the item pool: {:?}",
                    to_add.name
                );
            }
        }

        // No critical items left; pick one at random
        // At this point there *should* only be minor items left
        if let Some(item) = self.item_pool.get(0).map(|(item, _)| item.clone()) {
            if let Some(location) = self.find_item_home(&*item, rng) {
                println!("Placing {} in {}, to fill up space", item, location);
                self.place_item(&item, &location, &[]);
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
                    for (loc, _) in &self.closed_locations {
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
        }
        println!("Unassigned items:");
        for (item, _) in &self.item_pool {
            println!(" - {}", item);
        }
    }

    fn alloc_progress(&self) -> Result<String, std::fmt::Error> {
        use std::fmt::Write;
        let mut restrictions: Vec<_> = self.locations.iter().filter_map(|(l, _)| l.restriction).collect();
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

    pub fn allocate<R: Rng + ?Sized>(&mut self, rng: &mut R) {
        println!("Open locations: ");
        for loc in &self.open_locations {
            println!("  * {}", loc);
        }
        println!("Closed locations: ");
        for (loc, _) in &self.closed_locations {
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
                for (loc, req) in &self.closed_locations {
                    println!("  - {}", loc);
                    print!("    ");
                    let mut req = req.clone();
                    for (id, count) in &self.assigned_items {
                        req.assume_item(id, *count);
                    }
                    req.render();
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

    fn check_assignments(&self, assignments: &HashMap<Rc<Location>, Rc<ItemDef>>) {
        let mut graph = "digraph G {\n".to_string();
        let mut open_locations = HashSet::new();
        let mut acquired_items = HashMap::new();
        let mut new_locations: Vec<_> = self
            .locations
            .iter()
            .filter(|(_l, req)| req.satisfied(&acquired_items))
            .collect();
        let mut generations = vec![];
        let mut item_indices: HashMap<&Rc<ItemDef>, usize> = HashMap::new();
        let mut completed_flags: HashSet<Rc<Flag>> = HashSet::new();
        let mut new_flags: Conditionals<Flag> = vec![];
        while !new_locations.is_empty() || !new_flags.is_empty() {
            let mut this_gen = vec![];
            for (flag, req) in &new_flags {
                completed_flags.insert(flag.clone());
                graph.push_str(&format!(
                    r#"  "flag{}" [label="{}", shape="octagon"];"#,
                    flag.name, flag.name
                ));
                graph.push('\n');
                for sat in req.satisfied_by(&acquired_items) {
                    if !sat.show_in_graph {
                        continue;
                    }
                    let max_idx = *item_indices.get(&sat).unwrap();
                    for i in 1..=max_idx {
                        graph
                            .push_str(&format!(r#"  "flag{}" -> "{}{}";"#, flag.name, sat.name, i));
                        graph.push('\n');
                    }
                }
            }

            for (loc, req) in &new_locations {
                if let Some(item) = assignments.get(loc) {
                    if !item.show_in_graph {
                        continue;
                    }
                    let idx = item_indices.entry(item).or_default();
                    *idx += 1;
                    let idx = *idx;
                    graph.push_str(&format!(
                        r#"  "{}{}" [label="{}\n{}", shape="box"];"#,
                        item.name, idx, loc, item.name
                    ));
                    graph.push('\n');
                    for sat in req.satisfied_by(&acquired_items) {
                        if !sat.show_in_graph {
                            continue;
                        }
                        let max_idx = *item_indices.get(&sat).unwrap();
                        for i in 1..=max_idx {
                            if i == idx && &sat == item {
                                // Don't link an item to itself
                                continue;
                            }
                            graph.push_str(&format!(
                                r#"  "{}{}" -> "{}{}";"#,
                                item.name, idx, sat.name, i
                            ));
                            graph.push('\n');
                        }
                    }
                } else {
                    // graph.push_str(&format!(r#"  "{}" [label="{}\n{}", shape="box"];"#, item, loc.name, item));
                }
            }
            for (loc, _req) in new_locations {
                open_locations.insert(loc);
                if let Some(item) = assignments.get(loc) {
                    *acquired_items.entry(item.clone()).or_default() += 1;
                    this_gen.push((item.clone(), loc));
                } else {
                    eprintln!("Empty location?");
                }
            }
            for (flag, _) in new_flags {
                print!("{{{}}}, ", flag.name);
            }
            for (item, loc) in &this_gen {
                print!("{} ({}), ", item, loc);
            }
            println!();
            generations.push(this_gen);

            new_locations = self
                .locations
                .iter()
                .filter(|(l, _)| !open_locations.contains(l))
                .filter(|(_, req)| req.satisfied(&acquired_items))
                .collect();
            new_flags = self
                .flags
                .iter()
                .filter(|(f, _)| !completed_flags.contains(&**f))
                .filter(|(_, req)| req.satisfied(&acquired_items))
                .cloned()
                .collect();
        }
        graph.push('}');
        std::fs::write("graph.dot", graph.as_bytes());
        for (flag, _) in self.flags.iter().filter(|(f, _)| !completed_flags.contains(&**f)) {
            println!("Unsatisfied flag: {}", flag.name);
        }
    }
}
