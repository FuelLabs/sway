use structopt::{self, StructOpt};

#[derive(Debug, StructOpt)]
pub(crate) struct Command {}

pub(crate) fn exec(_command: Command) -> Result<(), String> {
    todo!()
}
