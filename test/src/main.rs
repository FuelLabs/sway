mod e2e_vm_tests;
mod ir_generation;

use clap::Parser;

#[derive(Parser)]
struct Cli {
    /// If specified, only run tests matching this regex
    #[clap(value_parser)]
    filter_regex: Option<regex::Regex>,

    /// If specified, skip tests matching this regex
    #[clap(long = "skip")]
    skip_regex: Option<regex::Regex>,

    /// If specified, skip until a test matches this regex
    #[clap(long = "skip-until")]
    skip_until_regex: Option<regex::Regex>,

    /// Intended for use in `CI` to ensure test lock files are up to date
    #[clap(long)]
    locked: bool,
}

fn main() {
    let cli = Cli::parse();

    e2e_vm_tests::run(
        cli.locked,
        cli.filter_regex.as_ref(),
        cli.skip_regex.as_ref(),
        cli.skip_until_regex.as_ref(),
    );
    ir_generation::run(cli.filter_regex.as_ref());
}
