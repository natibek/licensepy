use clap::Parser;
use std::cmp::min;
mod argparse;
use argparse::{Args, Commands};

mod check;
mod format;
mod metadata;
mod print_output;
mod utils;

use check::run_check;
use format::Formatter;

const MAX_THREADS: u8 = 32u8;

fn main() {
    let args = Args::parse();
    env_logger::init();
    match &args.command {
        Commands::Check {
            recursive,
            by_package,
            ignore_toml,
            silent,
            fail_print,
            num_threads,
        } => {
            let num_threads = min(MAX_THREADS, *num_threads);

            rayon::ThreadPoolBuilder::new()
                .num_threads(num_threads as usize)
                .build_global()
                .unwrap();
            run_check(*recursive, *by_package, *ignore_toml, *silent, *fail_print)
        }

        Commands::Format {
            files,
            licensee,
            license_year,
            silent,
            dry_run,
            num_threads,
        } => {
            let num_threads = min(MAX_THREADS, *num_threads);

            rayon::ThreadPoolBuilder::new()
                .num_threads(num_threads as usize)
                .build_global()
                .unwrap();

            let formatter = Formatter::new(files, licensee, license_year, *silent, *dry_run);
            formatter.format_files()
        }
    }
}
