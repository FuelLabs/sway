use structopt::{self, StructOpt};

/// Run Rust-based tests on current project.
#[derive(Debug, StructOpt)]
pub(crate) struct Command {}

pub(crate) fn exec(_command: Command) -> Result<(), String> {
    todo!()
}
