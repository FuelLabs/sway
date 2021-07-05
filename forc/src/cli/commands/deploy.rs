use structopt::{self, StructOpt};

use crate::ops::forc_deploy;

#[derive(Debug, StructOpt)]
pub struct Command {}

pub(crate) fn exec(command: Command) -> Result<(), String> {
    match forc_deploy::deploy(command) {
        Err(e) => Err(e.message),
        _ => Ok(()),
    }
}
