use clap::Parser;
use forc_tracing::{init_tracing_subscriber, println_error, TracingSubscriberOptions};

#[tokio::main]
async fn main() {
    let command = forc_client::cmd::Call::parse();

    // Initialize tracing with verbosity from command
    init_tracing_subscriber(TracingSubscriberOptions {
        verbosity: Some(command.verbosity),
        writer_mode: Some(command.output.clone().into()),
        regex_filter: Some("forc_tracing".to_string()),
        ..Default::default()
    });

    let operation = match command.validate_and_get_operation() {
        Ok(operation) => operation,
        Err(err) => {
            println_error(&err);
            std::process::exit(1);
        }
    };
    if let Err(err) = forc_client::op::call(operation, command).await {
        println_error(&format!("{}", err));
        std::process::exit(1);
    }
}
