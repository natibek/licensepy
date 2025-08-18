use crate::metadata::Metadata;
use colored::Colorize;
use std::collections::{HashMap, HashSet};

/// Print results of `licensepy check` grouped by package.
pub fn print_by_package(dependencies: Vec<Metadata>, recursive: bool, fail_print: bool) {
    let mut dep_map: HashMap<String, bool> = HashMap::new();

    for dep in &dependencies {
        dep_map.insert(dep.name.clone(), dep.bad_license);
    }
    let mut sorted_dep = dependencies.clone();
    sorted_dep.sort();

    for dep in &sorted_dep {
        let license = dep.license.join(" & ");

        if dep.bad_license {
            print!("{}  {} ({}) ", "✗".red().bold(), dep.name, license);
        } else if fail_print {
            continue;
        } else {
            print!("{}  {} ({}) ", "✔".cyan().bold(), dep.name, license);
        }

        if recursive && !dep.requirements.is_empty() {
            print!(" [ ");
            for req in &dep.requirements {
                if let Some(bad_req_license) = dep_map.get(req) {
                    if *bad_req_license {
                        print!("{}, ", req.red().bold());
                    } else if !fail_print {
                        print!("{}, ", req.bold())
                    }
                }
            }
            print!("]");
        }
        println!();
    }
}

/// Print results of `licensepy check` grouped by license.
pub fn print_by_license(
    dependencies: Vec<Metadata>,
    license_to_avoid: &[String],
    recursive: bool,
    fail_print: bool,
) {
    let mut license_map: HashMap<&str, Vec<Metadata>> = HashMap::new();
    let mut dep_map: HashMap<String, bool> = HashMap::new();
    let mut licenses: HashSet<&str> = HashSet::new();

    for dep in &dependencies {
        dep_map.insert(dep.name.clone(), dep.bad_license);
        for license in &dep.license {
            license_map.entry(license).or_default().push(dep.clone());
            licenses.insert(license);
        }
    }

    let mut sorted_licenses = licenses.into_iter().collect::<Vec<_>>();
    sorted_licenses.sort();

    for license in sorted_licenses {
        if let Some(deps) = license_map.get(license) {
            let num_deps = deps.len();
            if license_to_avoid.contains(&license.to_string()) {
                println!("---{} [{}]---  {}", license, num_deps, "✗".red().bold());
            } else if fail_print {
                continue;
            } else {
                println!("---{} [{}]---  {}", license, num_deps, "✔".cyan().bold());
            }
            for d in deps {
                print!("\t{}", d.name);
                if recursive && !d.requirements.is_empty() {
                    print!(" [ ");
                    for req in &d.requirements {
                        if let Some(&bad_req_license) = dep_map.get::<String>(req) {
                            if bad_req_license {
                                print!("{}, ", req.red().bold());
                            } else {
                                print!("{}, ", req.bold())
                            }
                        }
                    }
                    print!("]");
                }
                println!();
            }
        }
    }
}
