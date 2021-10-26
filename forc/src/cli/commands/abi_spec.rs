use crate::ops::forc_abi_spec;
use structopt::{self, StructOpt};

/// Compile the current or target project.
#[derive(Debug, StructOpt)]
pub struct Command {
    /// Path to the project, if not specified, current working directory will be used.
    #[structopt(short, long)]
    pub path: Option<String>,
    /// Offline mode, prevents Forc from using the network when managing dependencies.
    /// Meaning it will only try to use previously downloaded dependencies.
    #[structopt(long = "offline")]
    pub offline_mode: bool,
    /// Silent mode. Don't output any warnings or errors to the command line.
    #[structopt(long = "silent", short = "s")]
    pub silent_mode: bool,
    /// If set, outputs the resulting json to a file.
    #[structopt(short = "o")]
    pub json_outfile: Option<String>,
}

pub(crate) fn exec(command: Command) -> Result<(), String> {
    forc_abi_spec::generate_abi_spec(command)?;
    Ok(())
}
