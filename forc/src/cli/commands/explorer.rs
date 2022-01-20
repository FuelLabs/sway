use crate::ops::forc_explorer;
use structopt::StructOpt;

/// Run the network explorer.
#[derive(Debug, StructOpt)]
pub struct Command {
    /// The port number
    #[structopt(short = "p", long = "port", default_value = "3030")]
    pub port: String,
}

pub(crate) async fn exec(_command: Command) -> Result<(), String> {
    match forc_explorer::exec(_command).await {
        Err(e) => Err(e.to_string()),
        _ => Ok(()),
    }
}
