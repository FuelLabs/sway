use clap::Parser;
use forc_tracing::{init_tracing_subscriber, println_error};

#[tokio::main]
async fn main() {
    init_tracing_subscriber(Default::default());
    let command = forc_client::cmd::Call::parse();
    // Check if the print_functions flag is set
    if command.print_functions {
        if let Err(err) = forc_client::op::print_callable_functions(command).await {
            println_error(&format!("{}", err));
            std::process::exit(1);
        }
    } else {
        if let Err(err) = forc_client::op::call(command).await {
            println_error(&format!("{}", err));
            std::process::exit(1);
        }
    }
}
