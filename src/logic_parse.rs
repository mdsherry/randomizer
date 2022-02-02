use std::collections::HashSet;

use heck::ToSnakeCase;

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
            let var_name = bits[0].split(':').next().expect("Always at least one piece").to_snake_case();
            // println!("let flag_{} = cond_fact.add_flag(\"{}\", {});", var_name, bits[0], req_str);
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
            println!("let loc_{} = cond_fact.add_location(\"{}\", {}, {});", var_name, bits[0], req_str, category);
        }
    }
    for item in items {
        // println!("let {} = cond_fact.add_item(\"{}\", ItemCategory::Major);", item.to_snake_case(), item);
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Term<'a> {
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
        Term::Count(_, _) => "todo!()".to_string(),
    }
}

fn parse_reqs(mut s: &str) -> (Vec<Term>, &str) {
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