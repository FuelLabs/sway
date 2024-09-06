//! A forc plugin to start a fuel core instance, preconfigured for generic
//! usecases.
mod cmd;
mod local;
mod op;
mod pkg;
mod run;
mod testnet;

use clap::Parser;
use forc_tracing::{init_tracing_subscriber, println_error};

#[tokio::main]
async fn main() {
    init_tracing_subscriber(Default::default());
    let command = cmd::ForcNode::parse();
    if let Err(err) = op::run(command).await {
        println_error(&format!("{}", err));
        std::process::exit(1);
    }
}
