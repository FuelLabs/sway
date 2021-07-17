use structopt::{self, StructOpt};

#[derive(Debug, StructOpt)]
/// Run Rust-based tests on current project.
pub(crate) struct Command {}

pub(crate) fn exec(_command: Command) -> Result<(), String> {
    todo!()
}
