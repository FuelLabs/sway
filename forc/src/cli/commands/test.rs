use structopt::{self, StructOpt};

#[derive(Debug, StructOpt)]
pub(crate) struct Command {}

pub(crate) fn exec(command: Command) -> Result<(), String> {
    todo!()
}
