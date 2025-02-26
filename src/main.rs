use clap::Parser;
mod argparse;
use argparse::Args;

mod utils;
use utils::{ read_toml, get_python_version };

mod print_output;
use print_output::{ print_by_package, print_by_license };

mod metadata;
use metadata::Metadata;

mod licensepy;
use licensepy::{ get_package_dir, get_dist_directories, DistType };


fn main() {
    let args = Args::parse();
    let recursive: bool = args.recursive;
    let by_package: bool = args.by_package;

    let license_to_avoid: Vec<String> = if args.ignore_toml {
        Vec::new()
    } else {
        read_toml()
    };

    println!("Avoid {:?}", license_to_avoid);

    let python_version: [i32; 3] = get_python_version();
    let str_version = python_version
        .iter()
        .map(|n| format!("{}", n))
        .collect::<Vec<_>>()
        .join(".");

    println!("PYTHON VERSION {:}", str_version);
    let dist_dirs = get_dist_directories();
    println!("Dependencies stored at {:#?}.", dist_dirs);
    let package_dist: Vec<DistType> = dist_dirs
                    .into_iter()
                    .flat_map(get_package_dir)
                    .collect();
    // println!("{:?}", package_dist);
    println!();

    let dependencies: Vec<Metadata> = package_dist
                    .into_iter()
                    .map(|dist| dist.get_metadata(&python_version, recursive, &license_to_avoid))
                    .collect();

    let num_dep = dependencies.len();
    // println!("{:?}", dependencies);
    if by_package {
        print_by_package(dependencies, recursive);
    } else {
        print_by_license(dependencies, recursive, &license_to_avoid);
    }
    println!();
    println!("Found {} dependencies.", num_dep);
}



