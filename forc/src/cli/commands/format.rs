use structopt::{self, StructOpt};

use crate::ops::forc_fmt;

#[derive(Debug, StructOpt)]
/// Format all Sway files of the current project.
pub struct Command {
    #[structopt(short, long)]
    pub check: bool,
}

pub(crate) fn exec(command: Command) -> Result<(), String> {
    match forc_fmt::format(command) {
        Err(e) => Err(e.message),
        _ => Ok(()),
    }
}
