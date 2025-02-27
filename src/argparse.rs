use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    /// Find requirements for each dependency. Default is false.
    #[arg(short, long, default_value_t = false)]
    pub recursive: bool,

    /// Print output by package. Default is false which prints by license.
    #[arg(short, long, default_value_t = false)]
    pub by_package: bool,

    /// Ignore pyproject.toml file. Default if false.
    #[arg(short, long, default_value_t = false)]
    pub ignore_toml: bool,

    /// Don't print any outputs. Default if false.
    #[arg(short, long, default_value_t = false)]
    pub silent: bool,

    /// Print only fail. Default if false.
    #[arg(short, long, default_value_t = false)]
    pub fail_print: bool,
}
