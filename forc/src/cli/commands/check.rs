use crate::ops::forc_check;
use anyhow::Result;
use clap::Parser;

/// Check the current or target project and all of its dependencies for errors.
///
/// This will essentially compile the packages without performing the final step of code generation,
/// which is faster than running forc build.
#[derive(Debug, Default, Parser)]
pub struct Command {
    /// Path to the project, if not specified, current working directory will be used.
    #[clap(short, long)]
    pub path: Option<String>,
    /// Offline mode, prevents Forc from using the network when managing dependencies.
    /// Meaning it will only try to use previously downloaded dependencies.
    #[clap(long = "offline")]
    pub offline_mode: bool,
    /// Requires that the Forc.lock file is up-to-date. If the lock file is missing, or it
    /// needs to be updated, Forc will exit with an error
    #[clap(long)]
    pub locked: bool,
    /// Terse mode. Limited warning and error output.
    #[clap(long = "terse", short = 't')]
    pub terse_mode: bool,
    /// TODO remove this when we remove lazy statics from the compiler.
    /// This is just an implementation detail for improving LSP performance
    /// we let the lazy statics continually leak in order to allow for parallel
    /// project compilation
    #[clap(hide = true, long = "preserve-statics")]
    pub preserve_statics: bool,
}

pub(crate) fn exec(command: Command) -> Result<()> {
    let res = forc_check::check(command)?;
    if !res.is_ok() {
        anyhow::bail!("unable to type check");
    }
    Ok(())
}
