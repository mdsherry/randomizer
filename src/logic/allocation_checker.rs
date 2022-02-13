use std::{collections::{HashMap, HashSet}, rc::Rc};

use super::{Conditionals, Location, Flag, ItemDef};

pub struct AssignmentChecker {
    locations: Conditionals<Location>,
    flags: Conditionals<Flag>,
}
impl AssignmentChecker {
    pub fn new(locations: Conditionals<Location>, flags: Conditionals<Flag>) -> Self {
        AssignmentChecker { locations, flags }
    }

    
    pub fn check_assignments(&self, assignments: &HashMap<Rc<Location>, Rc<ItemDef>>) {
        let mut locations = self.locations.clone();
        let orig_locs: HashMap<_, _> = locations.iter().cloned().collect();
        let mut flags = self.flags.clone();
        let orig_flags: HashMap<_, _> = flags.iter().cloned().collect();
        let mut graph = "digraph G {\n".to_string();
        let mut open_locations = HashSet::new();
        let mut acquired_items = HashMap::new();
        let mut new_locations: Vec<_> = self
            .locations
            .iter()
            .filter(|(_l, req)| req.satisfied())
            .map(|(loc, _)| loc.clone())
            .collect();
        let mut generations = vec![];
        let mut item_indices: HashMap<&Rc<ItemDef>, usize> = HashMap::new();
        let mut completed_flags: HashSet<Rc<Flag>> = HashSet::new();
        let mut new_flags: Conditionals<Flag> = vec![];

        while !new_locations.is_empty() || !new_flags.is_empty() {
            let mut this_gen = vec![];
            for (flag, _req) in &new_flags {
                completed_flags.insert(flag.clone());
                graph.push_str(&format!(
                    r#"  "flag{}" [label="{}", shape="octagon"];"#,
                    flag.name, flag.name
                ));
                graph.push('\n');
                let orig_flag_req = &orig_flags[flag];
                let orig_flag_req = orig_flag_req.prune_sat(&acquired_items).map_err(|_| {
                    eprintln!("Flag not actually satisfied? {} {} {}", flag.name, _req, orig_flag_req);
                }).unwrap();
                let mut satisfiers = HashMap::new();
                orig_flag_req.min_sat(&acquired_items, &mut satisfiers);
                for (sat, max_idx) in satisfiers {
                    if !sat.show_in_graph {
                        continue;
                    }
                    // let max_idx = *item_indices.get(&sat).unwrap();
                    for i in 1..=max_idx {
                        graph
                            .push_str(&format!(r#"  "flag{}" -> "{}{}";"#, flag.name, sat.name, i));
                        graph.push('\n');
                    }
                }
            }

            for loc in &new_locations {
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
                    let orig_loc_req = &orig_locs[loc];
                    let orig_loc_req = orig_loc_req.prune_sat(&acquired_items).expect("New location not satisifed?");
                    let mut satisfiers = HashMap::new();
                    orig_loc_req.min_sat(&acquired_items, &mut satisfiers);
                    for (sat, max_idx) in satisfiers {
                        if !sat.show_in_graph {
                            continue;
                        }
                        // let max_idx = *item_indices.get(&sat).unwrap();
                        for i in 1..=max_idx {
                            if i == idx && sat == item {
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
            for loc in new_locations {
                open_locations.insert(loc.clone());
                if let Some(item) = assignments.get(&loc) {
                    *acquired_items.entry(item).or_default() += 1;
                    for (_, req) in &mut locations {
                        req.assume_item(item, 1);
                    }
                    for (_, req) in &mut flags {
                        req.assume_item(item, 1);
                    }
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

            new_locations = locations
                .iter()
                .filter(|(l, _)| !open_locations.contains(l))
                .filter(|(_, req)| req.satisfied())
                .map(|(loc, _)| loc.clone())
                .collect();
            new_flags = flags
                .iter()
                .filter(|(f, _)| !completed_flags.contains(&**f))
                .filter(|(_, req)| req.satisfied())
                .cloned()
                .collect();
        }
        graph.push('}');
        std::fs::write("graph.dot", graph.as_bytes());
        for (flag, _) in self
            .flags
            .iter()
            .filter(|(f, _)| !completed_flags.contains(&**f))
        {
            println!("Unsatisfied flag: {}", flag.name);
        }
    }
}
