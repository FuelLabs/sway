use forc_client::deploy::{cmd::DeployCommand, op::deploy};
use forc_tracing::init_tracing_subscriber;
use std::process;

use clap::Parser;

#[tokio::main]
pub async fn main() {
    init_tracing_subscriber(Default::default());
    let command = DeployCommand::parse();
    if let Err(err) = deploy(command).await {
        tracing::error!("Error: {:?}", err);
        process::exit(1);
    }
}
