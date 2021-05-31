use structopt::{self, StructOpt};

use crate::ops::forc_build;
#[derive(Debug, StructOpt)]
pub(crate) struct Command {
    #[structopt(short = "p")]
    pub path: Option<String>,
    /// Whether to compile to bytecode (false) or to print out the generated ASM (true).
    #[structopt(long = "asm")]
    pub asm: bool,
}

pub(crate) fn exec(command: Command) -> Result<(), String> {
    if command.asm {
        forc_build::print_asm(command.path)
    } else {
        forc_build::build(command.path);
        Ok(())
    }
}
