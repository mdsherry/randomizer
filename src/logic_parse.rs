use std::collections::{HashSet, HashMap};

use heck::ToSnakeCase;

use crate::logic::{Condition, ItemId, FlagId, LocationId};

pub fn parse_logic(s: &str) {
    let lines = s.lines()
        .map(|line| line.trim())
        .filter(|line| !(line.is_empty() || line.starts_with('#')));
    let mut items = HashSet::new();
    for line in lines {
        if line.starts_with('!') {
            continue;
        }
        
        let bits: Vec<_> = line.split(';').map(|s| s.trim()).collect();
        
        if bits.len() < 2 {
            println!("{:?}", bits);
            continue;

        }
        let location: Vec<_> = bits[0].split(':').collect();

        if bits[1] == "Helper" {
            let (reqs, _) = parse_reqs(bits[3]);
            get_items(&reqs, &mut items);
            let req_str = gen_reqs(&reqs);
            let var_name = bits[0].split(':').next().expect("Always at least one piece");
            // println!("let flag_{} = cond_fact.add_flag(\"{}\", {});", var_name, bits[0], req_str);
//             println!("
//   - name: {}
//     requirements: {}", var_name, bits[3]);
        } else if bits[1] == "Major" || bits[1] == "Minor"  || bits[1] == "DungeonItem" {
            let (reqs, _) = parse_reqs(bits[3]);
            get_items(&reqs, &mut items);
            let req_str = gen_reqs(&reqs);
            let var_name = location[0].to_snake_case();
            let category = if location.len() > 1 {
                format!("ItemCategory::Class({})", location[1])
            } else {
                format!("ItemCategory::{}", bits[1])
            };
            // println!("let loc_{} = cond_fact.add_location(\"{}\", {}, {});", var_name, bits[0], req_str, category);
                        print!("
  - name: {}
    category: {}
    requirements: {}", location[0], bits[1], bits[3]);
            if location.len() > 1 {
            println!("
    restriction: {}", location[1]);
            }
        }
    }
    for item in items {
        // println!("let {} = cond_fact.add_item(\"{}\", ItemCategory::Major);", item.to_snake_case(), item);
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Term<'a> {
    Lit(&'a str),
    And(Vec<Term<'a>>),
    Or(Vec<Term<'a>>),
    Count(u32, Vec<Term<'a>>)
}

fn get_items(terms: &[Term], collected: &mut HashSet<String>) {
    for term in terms {
        match term {
            Term::Lit(s) => if let Some(mut item) = s.strip_prefix("Items.") {
                item = item.split(':').next().expect("We always have at least *one* string piece");
                if !collected.contains(item) {
                    collected.insert(item.to_string());
                }
            },
            Term::And(terms) => get_items(terms, collected),
            Term::Or(terms) => get_items(terms, collected),
            Term::Count(_, terms) => get_items(terms, collected),
        }
    }
}

fn gen_reqs(terms: &[Term<'_>]) -> String {
    match terms {
        [] => "Condition::NoRequirements".into(),
        [ref a] => gen_req(a),
        _ => gen_req(&Term::And(terms.to_vec()))
    }
}

fn gen_req(term: &Term) -> String {
    match term {
        Term::Lit(s) => {
            if let Some(mut item) = s.strip_prefix("Items.") {
                let mut bits = item.split(':');
                item = bits.next().expect("We always have at least *one* string piece");
                if let Some(count) = bits.next() {
                    format!("Condition::Item({}, {})", item.to_snake_case(), count)
                } else {
                    item.to_snake_case()
                }
            } else if let Some(helper) = s.strip_prefix("Helpers.") {
                format!("flag_{}", helper.to_snake_case())
            } else if let Some(location) = s.strip_prefix("Locations.") {
                format!("loc_{}", location.to_snake_case())
            } else {
                eprintln!("?? {}", s);
                s.to_snake_case()
            }
        },
        Term::And(terms) => {
            let mut s = "cond_and!(".to_string();
            let mut first = true;
            for term in terms {
                if !first {
                    s.push_str(", ");
                }
                first = false;
                s.push_str(&gen_req(term));                
            }
            s.push(')');
            s
        },
        Term::Or(terms) => 
        {
            let mut s = "cond_or!(".to_string();
            let mut first = true;
            for term in terms {
                if !first {
                    s.push_str(", ");
                }
                first = false;
                s.push_str(&gen_req(term));
            }
            s.push(')');
            s
        },
        Term::Count(threshold, terms) => {
            let mut s = format!("Condition::AtLeast({}, vec![", threshold);
            let mut first = true;
            for term in terms {
                if !first {
                    s.push_str(", ");
                }
                first = false;
                match term {
                    Term::Lit(lit) => {
                        let mut it = lit.split(':');
                        let name = it.next().unwrap();
                        s.push('(');
                        if let Some(item) = name.strip_prefix("Items.") {
                            s.push_str(&item.to_snake_case());
                        }
                        let weight = it.next();
                        s.push_str(", ");
                        s.push_str(weight.unwrap_or("1"));
                        s.push(')');

                    }
                    _ => panic!("Count terms can only contain item literals")
                }
            }
            s.push_str("])");
            s
        }
    }
}

pub fn parse_reqs(mut s: &str) -> (Vec<Term>, &str) {
    s = s.trim();
    if s.is_empty() {
        return (vec![], s)
    }
    let mut terms = vec![];
    while !s.is_empty() {
        // println!("{}", s);
        if s.starts_with("(|") {
            let (subterms, new_s) = parse_reqs(&s[2..]);
            s = new_s;
            terms.push(Term::Or(subterms));
        } else if s.starts_with("(&") {
            let (subterms, new_s) = parse_reqs(&s[2..]);
            s = new_s;
            terms.push(Term::And(subterms));
        } else if s.starts_with("(+") {
            if let Some(comma_pos) = s.find(',') {
                // println!("{}", &s[2..comma_pos]);
                let count_res = s[2..comma_pos].trim().parse::<u32>();
                
                let count = match count_res {
                    Ok(c) => c,
                    Err(_) => 0
                };
                let (subterms, new_s) = parse_reqs(&s[comma_pos + 1..]);
                s = new_s;
                terms.push(Term::Count(count, subterms));
            }
        } else {
            if let Some(end) = s.find(|c| matches!(c, ',' | ')')) {
                let term = s[..end].trim();
                if !term.is_empty() {
                    terms.push(Term::Lit(term));
                }                
                s = &s[end..];
            } else {
                terms.push(Term::Lit(s.trim()));
                s = &s[s.len()..];
            }
            s = s.trim_start_matches(',');
            if let Some(s) = s.strip_prefix(')') {
                return (terms, s.trim());
            }
            
        }
        s = s.trim();
    }

    (terms, s)
}

#[test]
fn parse_palace() {
    let s = "Locations.AccessUpperClouds, Helpers.CanSplit3, (|Items.RocsCape, Items.BombBag, Items.GustJar, Helpers.HasBoomerang, Helpers.HasBow)";
    let (terms, remaining) = parse_reqs(s);
    assert_eq!(remaining, "");
    use Term::*;
    assert_eq!(terms, vec![Lit("Locations.AccessUpperClouds"), Lit("Helpers.CanSplit3"), Or(vec![Lit("Items.RocsCape"), Lit("Items.BombBag"), Lit("Items.GustJar"), Lit("Helpers.HasBoomerang"), Lit("Helpers.HasBow")])]);
}


pub fn gen_reqs2(terms: &[Term<'_>], items: &HashMap<&str, ItemId>, flags: &HashMap<&str, FlagId>, locations: &HashMap<&str, LocationId>, parameters: &HashSet<&str>) -> Result<Condition, LogicParseError> {
    match terms {
        [] => Ok(Condition::NoRequirements),
        [ref a] => gen_req2(a, items, flags, locations, parameters),
        _ => gen_req2(&Term::And(terms.to_vec()), items, flags, locations, parameters)
    }
}

use thiserror::Error;
#[derive(Debug, Error)]
pub enum LogicParseError {
    #[error("Unknown item {0}")]
    UnrecognizedItem(String),
    #[error("Unknown location {0}")]
    UnrecognizedLocation(String),
    #[error("Unknown flag {0}")]
    UnrecognizedFlag(String),
    #[error("Unknown parameter {0}")]
    UnrecognizedParameter(String),
    #[error("Name is not an item, flag or location: {0}")]
    UnrecognizedName(String),
    #[error("Name was not unique; qualify with Items., Helpers., Locations. or Parameters.: {0}")]
    AmbiguousName(String),
    #[error("Thresholds require item literals, not more complex expressions")]
    ThresholdRequireItems,
}

fn add_item(item: &str, items: &HashMap<&str, ItemId>) -> Result<Condition, LogicParseError> {
    let mut bits = item.split('*');
    let item = bits.next().expect("We always have at least *one* string piece");
    let id = items.get(item);
    let id = id.ok_or_else(|| LogicParseError::UnrecognizedItem(item.to_string()))?;
    Ok(if let Some(count) = bits.next() {
        Condition::Item(*id, count.parse::<usize>().unwrap())
    } else {
        Condition::Item(*id, 1)
    })
}
fn add_location(loc: &str, locations: &HashMap<&str, LocationId>) -> Result<Condition, LogicParseError> {
    let id = locations.get(loc);
    let id = id.ok_or_else(|| LogicParseError::UnrecognizedLocation(loc.to_string()))?;
    Ok(Condition::Location(*id))
}
fn add_flag(flag: &str, flags: &HashMap<&str, FlagId>) -> Result<Condition, LogicParseError> {
    let id = flags.get(flag);
    let id = id.ok_or_else(|| LogicParseError::UnrecognizedFlag(flag.to_string()))?;
    Ok(Condition::Flag(*id))
}

fn add_parameter(name: &str, parameters: &HashSet<&str>) -> Result<Condition, LogicParseError> {
    if parameters.contains(name) {
        Ok(Condition::Parameter(name.into()))
    } else {
        Err(LogicParseError::UnrecognizedParameter(name.into()))
    }
}

fn gen_req2(term: &Term, items: &HashMap<&str, ItemId>, flags: &HashMap<&str, FlagId>, locations: &HashMap<&str, LocationId>, parameters: &HashSet<&str>) -> Result<Condition, LogicParseError> {
    Ok(match term {
        Term::Lit(s) => {
            if let Some(item) = s.strip_prefix("Items.") {
                add_item(item, items)?
            } else if let Some(helper) = s.strip_prefix("Helpers.") {
                add_flag(helper, flags)?
            } else if let Some(location) = s.strip_prefix("Locations.") {
                add_location(location, locations)?
            } else if let Some(parameter) = s.strip_prefix("Parameter.") {
                add_parameter(parameter, parameters)?
            } else {
                match (add_item(s, items), add_flag(s, flags), add_location(s, locations), add_parameter(s, parameters)) {
                    (Ok(c), Err(_), Err(_), Err(_)) => c,
                    (Err(_), Ok(c), Err(_), Err(_)) => c,
                    (Err(_), Err(_), Ok(c), Err(_)) => c,
                    (Err(_), Err(_), Err(_), Ok(c)) => c,
                    (Err(_), Err(_), Err(_), Err(_)) => return Err(LogicParseError::UnrecognizedName(s.to_string())),
                    (_, _, _, _) => return Err(LogicParseError::AmbiguousName(s.to_string()))
                }
            }
        },
        Term::And(terms) => {
            Condition::And(terms.iter().map(|term| gen_req2(term, items, flags, locations, parameters)).collect::<Result<_, _>>()?)
        },
        Term::Or(terms) => 
        {
            Condition::Or(terms.iter().map(|term| gen_req2(term, items, flags, locations, parameters)).collect::<Result<_, _>>()?)
        },
        Term::Count(threshold, terms) => {
            let items = terms.iter().map(|term| match term {
                Term::Lit(lit) => {
                    let mut it = lit.split('*');
                    let name = it.next().unwrap();
                    
                    let id = (if let Some(item) = name.strip_prefix("Items.") {
                        items.get(item)
                    } else {
                        items.get(name)
                    }).ok_or_else(|| LogicParseError::UnrecognizedItem(name.to_string()))?;
                    let weight = it.next().map(|s| s.parse::<usize>().unwrap());
                    Ok((*id, weight.unwrap_or(1)))
                }
                _ => Err(LogicParseError::ThresholdRequireItems)
            }).collect::<Result<Vec<_>, _>>();
            Condition::AtLeast(*threshold as _, items?)
        }
    })
}