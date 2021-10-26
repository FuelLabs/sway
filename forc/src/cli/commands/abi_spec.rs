use crate::ops::forc_abi_spec;
use structopt::{self, StructOpt};

/// Compile the current or target project.
#[derive(Debug, StructOpt)]
pub struct Command {

}

pub(crate) fn exec(command: Command) -> Result<(), String> {
    forc_abi_spec::generate_abi_spec(command)?;
    Ok(())
}
