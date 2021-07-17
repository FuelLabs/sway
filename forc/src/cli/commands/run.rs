use crate::ops::forc_run;
use structopt::{self, StructOpt};

#[derive(Debug, StructOpt)]
/// Run script project.
pub struct Command {
    #[structopt(short, long)]
    pub data: Option<String>,

    /// Path to the project, if not specified, current working directory will be used.
    #[structopt(short, long)]
    pub path: Option<String>,

    #[structopt(long)]
    pub dry_run: bool,
}

pub(crate) async fn exec(command: Command) -> Result<(), String> {
    match forc_run::run(command).await {
        Err(e) => Err(e.message),
        _ => Ok(()),
    }
}
