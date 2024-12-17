use clap::Parser;
use forc_tracing::{init_tracing_subscriber, println_error};

#[tokio::main]
async fn main() {
    init_tracing_subscriber(Default::default());
    let command = forc_client::cmd::Call::parse();
    if let Err(err) = forc_client::op::call(command).await {
        println_error(&format!("{}", err));
        std::process::exit(1);
    }
}
