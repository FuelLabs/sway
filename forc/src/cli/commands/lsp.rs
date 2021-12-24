use structopt::StructOpt;
use sway_server::start;
/// Run the LSP server.
#[derive(Debug, StructOpt)]
pub(crate) struct Command {}

pub(crate) async fn exec(_command: Command) -> Result<(), String> {
    println!(
        "Running the {} language server...",
        sway_utils::constants::LANGUAGE_NAME
    );
    Ok(start().await)
}
