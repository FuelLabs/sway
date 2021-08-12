use crate::ops::forc_update;
use structopt::{self, StructOpt};

/// Update dependencies in the Forc dependencies directory.
#[derive(Debug, StructOpt)]
pub struct Command {
    /// Path to the project, if not specified, current working directory will be used.
    #[structopt(short, long)]
    pub path: Option<String>,

    /// Dependency to be updated.
    /// If not set, all dependencies will be updated.
    #[structopt(short = "d")]
    pub target_dependency: Option<String>,

    /// Checks if the dependencies have newer versions.
    /// Won't actually perform the update, will output which
    /// ones are up-to-date and outdated.
    #[structopt(short, long)]
    pub check: bool,
}

pub(crate) async fn exec(command: Command) -> Result<(), String> {
    match forc_update::update(command).await {
        Ok(_) => Ok(()),
        Err(e) => Err(format!("couldn't update dependencies: {}", e)),
    }
}
