use structopt::{self, StructOpt};

use crate::ops::forc_doc;

#[derive(Debug, StructOpt)]
pub struct Command {
    /// Path to the project, if not specified, current working directory will be used.
    #[structopt(short, long)]
    pub path: Option<String>,
}

pub(crate) fn exec(command: Command) -> Result<(), String> {
    match forc_doc::doc(command) {
        Err(e) => Err(e.message),
        _ => Ok(()),
    }
}
