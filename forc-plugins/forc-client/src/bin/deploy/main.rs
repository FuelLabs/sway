use forc_client::ops::deploy::{cmd::DeployCommand, op::deploy};
use forc_util::init_tracing_subscriber;
use std::process;

use clap::Parser;

#[tokio::main]
pub async fn main() {
    init_tracing_subscriber(None);
    let command = DeployCommand::parse();
    if let Err(err) = deploy(command).await {
        eprintln!("Error: {:?}", err);
        process::exit(1);
    }
}
