use structopt::{self, StructOpt};

use crate::ops::forc_build;
#[derive(Debug, StructOpt)]
pub struct Command {
    #[structopt(short = "p")]
    pub path: Option<String>,
    /// Whether to compile to bytecode (false) or to print out the generated ASM (true).
    #[structopt(long = "print-asm")]
    pub print_asm: bool,
    /// Whether to output a binary file representing the script bytes
    #[structopt(short = "o")]
    pub binary_outfile: Option<String>,

    /// Offline mode, prevents Forc from using the network when managing dependencies.
    /// Meaning it will only try to use previously downloaded dependencies.
    #[structopt(long = "offline")]
    pub offline_mode: bool,
}

pub(crate) fn exec(command: Command) -> Result<(), String> {
    forc_build::build(command)?;
    Ok(())
}
