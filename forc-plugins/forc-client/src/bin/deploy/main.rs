use clap::Parser;
use forc_tracing::init_tracing_subscriber;

#[tokio::main]
async fn main() {
    init_tracing_subscriber(Default::default());
    let command = forc_client::cmd::Deploy::parse();
    if let Err(err) = forc_client::op::deploy(command).await {
        tracing::error!("Error: {:?}", err);
        std::process::exit(1);
    }
}
