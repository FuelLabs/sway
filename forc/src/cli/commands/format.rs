use crate::ops::forc_fmt;
use clap::Parser;

/// Format all Sway files of the current project.
#[derive(Debug, Parser)]
pub struct Command {
    /// Run in 'check' mode.
    /// Exits with 0 if input is formatted correctly.
    /// Exits with 1 and prints a diff if formatting is required.
    #[clap(short, long)]
    pub check: bool,
}

// todo: add formatting options in the command line
pub(crate) fn exec(command: Command) -> Result<(), String> {
    match forc_fmt::format(command) {
        Err(e) => Err(e.message),
        _ => Ok(()),
    }
}
