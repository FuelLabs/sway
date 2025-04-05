use clap::Parser;
use forc_tracing::{init_tracing_subscriber, println_error};

#[tokio::main]
async fn main() {
    init_tracing_subscriber(Default::default());
    let command = forc_client::cmd::Call::parse();
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
