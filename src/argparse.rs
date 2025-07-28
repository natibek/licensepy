use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(version, about, long_about = None)]
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
    },

    /// Run license header formatter on files.
    Format {
        /// Files to format
        #[arg()]
        files: Vec<String>,

        /// Find requirements for each dependency. Default is false.
        #[arg(short, long)]
        licensee: Option<String>,

        /// License year
        #[arg(short = 'y', long)]
        license_year: Option<u16>,

        /// Number of threads to use
        #[arg(short = 'j', long, default_value_t = 1)]
        num_threads: u8,
    },
}
