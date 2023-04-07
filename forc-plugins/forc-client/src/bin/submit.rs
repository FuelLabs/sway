use clap::Parser;
use forc_tracing::{init_tracing_subscriber, TracingSubscriberOptions};

#[tokio::main]
async fn main() {
    init_tracing_subscriber(TracingSubscriberOptions {
        ansi: Some(true),
        ..Default::default()
    });
    let command = forc_client::cmd::Submit::parse();
    if let Err(err) = forc_client::op::submit(command).await {
        tracing::error!("Error: {:?}", err);
        std::process::exit(1);
    }
}
