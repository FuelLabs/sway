use clap::Parser;
use forc_client::ops::deploy::{cmd::Deploy, op::deploy};
use forc_util::init_tracing_subscriber;
use std::process;

#[tokio::main]
pub async fn main() {
    init_tracing_subscriber();
    let command = Deploy::parse();
    if let Err(err) = deploy(command).await {
        eprintln!("Error: {:?}", err);
        process::exit(1);
    }
}
