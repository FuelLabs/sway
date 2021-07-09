use structopt::{self, StructOpt};

use crate::ops::forc_update;
#[derive(Debug, StructOpt)]
pub struct Command {
    #[structopt(short = "p")]
    pub path: Option<String>,

    // Dependency to be updated.
    // If `d` isn't specified, all dependencies will be updated.
    #[structopt(short = "d")]
    pub target_dependency: Option<String>,
}

pub(crate) fn exec(command: Command) -> Result<(), String> {
    match forc_update::update(command) {
        Ok(_) => Ok(()),
        Err(e) => Err(format!("couldn't update dependencies: {}", e)),
    }
}
