//! A sample `forc` plugin example.
//!
//! Once installed and available via `PATH`, can be executed via `forc gm`.

use clap::Parser;

#[derive(Debug, Parser)]
#[clap(name = "forc-gm", about = "Sample Forc plugin", version)]
struct App {
    // Your name which Forc should greet!
    #[clap(short = 'n', long = "name", default_value = "")]
    pub name: String,
    #[clap(subcommand)]
    pub subcmd: Option<Subcommand>,
}

#[derive(Debug, Parser)]
enum Subcommand {
    // Say 'gm' to Sway Language!
    Sway,
}

fn main() {
    let app = App::parse();

    match app.subcmd {
        Some(Subcommand::Sway) => greet_sway(),
        None => run(app),
    }
}

fn greet_sway() {
    println!("gn from Fuel!");
}

fn run(app: App) {
    let App { name, .. } = app;

    if name.is_empty() {
        println!("gn!");
    } else {
        println!("gn {}!", &name);
    }
}
