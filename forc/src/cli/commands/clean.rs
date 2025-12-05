use crate::ops::forc_clean;
use clap::Parser;
use forc_types::ForcResult;

forc_types::cli_examples! {
    crate::cli::Opt {
        [Clean project => "forc clean"]
        [Clean project with a custom path => "forc clean --path <PATH>"]
    }
}

/// Removes the default forc compiler output artifact directory, i.e. `<project-name>/out`.
#[derive(Debug, Parser)]
#[clap(bin_name = "forc clean", version, after_help = help())]
pub struct Command {
    /// Path to the project, if not specified, current working directory will be used.
    #[clap(short, long)]
    pub path: Option<String>,
}

pub fn exec(command: Command) -> ForcResult<()> {
    forc_clean::clean(command)?;
    Ok(())
}
