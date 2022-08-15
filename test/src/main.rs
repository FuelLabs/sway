mod e2e_vm_tests;
mod ir_generation;

use clap::Parser;

#[derive(Parser)]
struct Cli {
    /// If specified, only run tests matching this regex
    #[clap(value_parser)]
    filter_regex: Option<regex::Regex>,

    /// Intended for use in `CI` to ensure test lock files are up to date
    #[clap(long)]
    locked: bool,
}

fn main() {
    let cli = Cli::parse();

    e2e_vm_tests::run(cli.locked, cli.filter_regex.as_ref());
    ir_generation::run(cli.filter_regex.as_ref());
}
