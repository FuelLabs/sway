mod cli;
mod deploy;

use std::process;

use clap::Parser;

fn main() {
    let args = cli::Deploy::parse();

    if let Err(err) = args.exec() {
        eprintln!("Error: {:?}", err);

        process::exit(1);
    }
}
