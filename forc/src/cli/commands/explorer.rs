use crate::ops::forc_explorer;
use structopt::StructOpt;

/// Run the network explorer.
#[derive(Debug, StructOpt)]
pub struct Command {
    /// The port number
    #[structopt(short = "p", long = "port", default_value = "3030")]
    pub port: String,
    #[structopt(subcommand)] // Note that we mark a field as a subcommand
    pub clean: Option<CleanCommand>,
}

#[derive(Debug, StructOpt)]
pub enum CleanCommand {
    Clean,
}

pub(crate) async fn exec(_command: Command) -> Result<(), String> {
    match forc_explorer::exec(_command).await {
        Err(e) => Err(e.to_string()),
        _ => Ok(()),
    }
}
