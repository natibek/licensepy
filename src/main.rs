use ::colored::Colorize;
use std::process::exit;

use clap::Parser;
mod argparse;
use argparse::Args;

mod utils;
use utils::{get_python_version, read_toml};

mod print_output;
use print_output::{print_by_license, print_by_package};

mod metadata;
use metadata::Metadata;

mod licensepy;
use licensepy::{get_dist_directories, get_package_dir, DistType};

fn main() {
    let args = Args::parse();
    let recursive: bool = args.recursive;
    let by_package: bool = args.by_package;

    let license_to_avoid: Vec<String> = if args.ignore_toml {
        Vec::new()
    } else {
        read_toml()
    };

    let python_version: [i32; 3] = get_python_version();
    let str_version = python_version
        .iter()
        .map(|n| format!("{}", n))
        .collect::<Vec<_>>()
        .join(".");

    let dist_dirs = get_dist_directories();

    if !args.silent {
        println!("Avoid {:?}", license_to_avoid);
        println!("PYTHON VERSION {:}", str_version);
        println!("Dependencies stored at {:#?}.", dist_dirs);
        println!();
    }

    let package_dist: Vec<DistType> = dist_dirs.into_iter().flat_map(get_package_dir).collect();
    // println!("{:?}", package_dist);

    let dependencies: Vec<Metadata> = package_dist
        .into_iter()
        .map(|dist| dist.get_metadata(&python_version, recursive, &license_to_avoid))
        .collect();

    let num_dep = dependencies.len();
    // println!("{:?}", dependencies);
    let num_bad_license: i32 = dependencies
        .iter()
        .filter(|dep| dep.bad_license)
        .count()
        .try_into()
        .unwrap();

    if !args.silent {
        if by_package {
            print_by_package(dependencies, recursive, args.fail_print);
        } else {
            print_by_license(dependencies, &license_to_avoid, recursive, args.fail_print);
        }
        println!();
        println!("Found {} total dependencies.", num_dep.to_string().cyan());
        println!(
            "Found {} dependencies with licenses to avoid.",
            num_bad_license.to_string().cyan()
        );
    }

    exit(num_bad_license);
}
