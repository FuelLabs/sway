use forc_client::ops::run::{cmd::Run, op::run};
use forc_util::init_tracing_subscriber;
use std::process;

use clap::Parser;

#[tokio::main]
async fn main() {
    init_tracing_subscriber();
    let command = Run::parse();
    if let Err(err) = run(command).await {
        eprintln!("Error: {:?}", err);
        process::exit(1);
    }
}
