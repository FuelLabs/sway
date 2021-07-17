use crate::ops::forc_run;
use structopt::{self, StructOpt};

#[derive(Debug, StructOpt)]
/// Run script project.
/// Crafts a script transaction then sends it to a running node.
pub struct Command {
    /// Hex string of data to input to script.
    #[structopt(short, long)]
    pub data: Option<String>,

    /// Path to the project, if not specified, current working directory will be used.
    #[structopt(short, long)]
    pub path: Option<String>,

    /// Only craft transaction and print it out.
    #[structopt(long)]
    pub dry_run: bool,
}

pub(crate) async fn exec(command: Command) -> Result<(), String> {
    match forc_run::run(command).await {
        Err(e) => Err(e.message),
        _ => Ok(()),
    }
}
