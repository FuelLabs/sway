use clap::Parser;
use forc_diagnostic::{init_tracing_subscriber, println_error, TracingSubscriberOptions};

#[derive(Parser, Debug)]
#[clap(name = "forc-debug", version)]
/// Forc plugin for the Sway DAP (Debug Adapter Protocol) implementation.
pub struct Opt {
    /// The URL of the Fuel Client GraphQL API
    #[clap(default_value = "http://127.0.0.1:4000/graphql")]
    pub api_url: String,
    /// Start the DAP server
    #[clap(short, long)]
    pub serve: bool,
}

#[tokio::main]
async fn main() {
    init_tracing_subscriber(TracingSubscriberOptions::default());
    let config = Opt::parse();

    let result = if config.serve {
        forc_debug::server::DapServer::default().start()
    } else {
        forc_debug::cli::start_cli(&config.api_url).await
    };

    if let Err(err) = result {
        println_error(&format!("{err}"));
        std::process::exit(1);
    }
}
