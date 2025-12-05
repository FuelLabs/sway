use clap::Parser;
use forc_diagnostic::{init_tracing_subscriber, println_error};

#[tokio::main]
async fn main() {
    init_tracing_subscriber(Default::default());
    let command = forc_client::cmd::Deploy::parse();
    if let Err(err) = forc_client::op::deploy(command).await {
        println_error(&format!("{err}"));
        std::process::exit(1);
    }
}
