use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(version, about, long_about = None)]
/// licensepy: Python project dependency license check and license header checking/formatting tool.
pub struct Args {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Run license check on project dependencies.
    Check {
        /// Find requirements for each dependency. Default is false.
        #[arg(short, long, default_value_t = false)]
        recursive: bool,

        /// Print output by package. Default is false which prints by license.
        #[arg(short = 'p', long, default_value_t = false)]
        by_package: bool,

        /// Ignore pyproject.toml file. Default if false.
        #[arg(short, long, default_value_t = false)]
        ignore_toml: bool,

        /// Don't print any outputs. Default if false.
        #[arg(short, long, default_value_t = false)]
        silent: bool,

        /// Print only fail. Default if false.
        #[arg(short, long, default_value_t = false)]
        fail_print: bool,

        /// Number of threads to use. Max is 32.
        #[arg(short = 'j', long, default_value_t = 1)]
        num_threads: u8,
    },

    /// Run license header formatter.
    Format {
        /// Files to run license header formatter on. If empty, will
        /// search for all python source code files starting current directory
        /// recursively exluding searches in directories starting with ".".
        #[arg()]
        files: Vec<String>,

        /// Licensee. Has precedence over value from config.
        #[arg(short, long)]
        licensee: Option<String>,

        /// License year. Has precedence over value from config.
        #[arg(short = 'y', long)]
        license_year: Option<u16>,

        /// Don't print any outputs. Default if false.
        #[arg(short, long, default_value_t = false)]
        silent: bool,

        /// Don't run formatter. Only print outputs. Default if false.
        #[arg(short, long, default_value_t = false)]
        dry_run: bool,

        /// Number of threads to use. Default is 1. Max is 32.
        #[arg(short = 'j', long, default_value_t = 1)]
        num_threads: u8,
    },
}
