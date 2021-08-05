use structopt::{self, StructOpt};

use crate::ops::forc_doc;

#[derive(Debug, StructOpt)]
pub struct Command {}

pub(crate) fn exec(command: Command) -> Result<(), String> {
    match forc_doc::doc(command) {
        Err(e) => Err(e.message),
        _ => Ok(()),
    }
}
