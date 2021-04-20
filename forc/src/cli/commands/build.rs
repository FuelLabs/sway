use structopt::{self, StructOpt};

use crate::ops::forc_build;
#[derive(Debug, StructOpt)]
pub(crate) struct Command {
    #[structopt(short = "p")]
    pub path: Option<String>,
}

pub(crate) fn exec(command: Command) -> Result<(), String> {
    forc_build::build(command.path)
}
