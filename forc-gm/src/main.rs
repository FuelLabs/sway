//! A sample `forc` plugin that greets you!
//!
//! Once installed and available via `PATH`, can be executed via `forc gm`.

use clap::Parser;

#[derive(Debug, Parser)]
#[clap(
    name = "forc-gm",
    about = "Sample Forc plugin that greets you!",
    version
)]
struct App {
    #[clap(subcommand)]
    pub subcmd: Option<Subcommand>,
}

#[derive(Debug, Parser)]
enum Subcommand {
    /// Say 'gm' to Fuel!
    Fuel,
}

fn main() {
    let app = App::parse();

    match app.subcmd {
        Some(Subcommand::Fuel) => greet_fuel(),
        None => greet(),
    }
}

fn greet_fuel() {
    println!("gn from Fuel!");
}

fn greet() {
    println!("gn!");
}
