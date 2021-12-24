use structopt::StructOpt;
use sway_server::start;
/// Run the LSP server.
#[derive(Debug, StructOpt)]
pub(crate) struct Command {}

pub(crate) async fn exec(_command: Command) -> Result<(), String> {
    Ok(start().await)
}
