use clap::Parser;
mod argparse;
use argparse::{Args, Commands};

mod check;
mod format;
mod metadata;
mod print_output;
mod utils;

use check::run_check;
use format::run_format;

fn main() {
    let args = Args::parse();

    match &args.command {
        Commands::Check {
            recursive,
            by_package,
            ignore_toml,
            silent,
            fail_print,
        } => {
            run_check(*recursive, *by_package, *ignore_toml, *silent, *fail_print);
        }
        Commands::Format {
            files,
            licensee,
            license_year,
            num_threads,
        } => run_format(files, licensee, license_year, num_threads),
    }
}
