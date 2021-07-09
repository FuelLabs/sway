use structopt::{self, StructOpt};

use crate::ops::forc_dep_check;
#[derive(Debug, StructOpt)]
pub struct Command {
    #[structopt(short = "p")]
    pub path: Option<String>,

    // Dependency to be checked.
    // If `d` isn't specified, all dependencies will be checked.
    #[structopt(short = "d")]
    pub target_dependency: Option<String>,
}

pub(crate) async fn exec(command: Command) -> Result<(), String> {
    match forc_dep_check::check(command).await {
        Ok(_) => Ok(()),
        Err(e) => Err(format!("couldn't check dependencies: {}", e)),
    }
}
