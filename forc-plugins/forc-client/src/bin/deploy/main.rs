pub mod cli;
pub mod deploy;

use forc_util::init_tracing_subscriber;
use std::process;

use clap::Parser;

#[tokio::main]
pub async fn main() {
    init_tracing_subscriber();
    let args = cli::Deploy::parse();

    if let Err(err) = args.exec().await {
        eprintln!("Error: {:?}", err);

        process::exit(1);
    }
}
