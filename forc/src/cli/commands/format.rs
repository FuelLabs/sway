use structopt::{self, StructOpt};

use crate::ops::forc_fmt;

#[derive(Debug, StructOpt)]
/// Format all Sway files of the current project.
pub struct Command {
    /// Run in 'check' mode.
    /// Exits with 0 if input is formatted correctly.
    /// Exits with 1 and prints a diff if formatting is required.
    #[structopt(short, long)]
    pub check: bool,
}

pub(crate) fn exec(command: Command) -> Result<(), String> {
    match forc_fmt::format(command) {
        Err(e) => Err(e.message),
        _ => Ok(()),
    }
}
