use crate::ops::forc_abi_json;
use anyhow::Result;
use clap::Parser;

/// Output the JSON associated with the ABI.
#[derive(Debug, Default, Parser)]
pub struct Command {
    /// Path to the project, if not specified, current working directory will be used.
    #[clap(short, long)]
    pub path: Option<String>,
    /// If set, outputs a json file representing the output json abi.
    #[clap(short = 'o')]
    pub json_outfile: Option<String>,
    /// Offline mode, prevents Forc from using the network when managing dependencies.
    /// Meaning it will only try to use previously downloaded dependencies.
    #[clap(long = "offline")]
    pub offline_mode: bool,
    /// Silent mode. Don't output any warnings or errors to the command line.
    #[clap(long = "silent", short = 's')]
    pub silent_mode: bool,
    /// By default the JSON for ABIs is formatted for human readability. By using this option JSON
    /// output will be "minified", i.e. all on one line without whitespace.
    #[clap(long)]
    pub minify: bool,
}

pub(crate) fn exec(command: Command) -> Result<()> {
    forc_abi_json::build(command)?;
    Ok(())
}
