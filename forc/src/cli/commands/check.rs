use crate::ops::forc_check;
use anyhow::Result;
use clap::Parser;
use sway_core::{decl_engine::DeclEngine, BuildTarget, Engines, TypeEngine};

/// Check the current or target project and all of its dependencies for errors.
///
/// This will essentially compile the packages without performing the final step of code generation,
/// which is faster than running forc build.
#[derive(Debug, Default, Parser)]
pub struct Command {
    /// Build target to use for code generation.
    #[clap(value_enum, default_value_t=BuildTarget::default(), alias="target")]
    pub build_target: BuildTarget,
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
    /// Disable checking unit tests.
    #[clap(long = "disable-tests")]
    pub disable_tests: bool,
}

pub(crate) fn exec(command: Command) -> Result<()> {
    let type_engine = TypeEngine::default();
    let decl_engine = DeclEngine::default();
    let engines = Engines::new(&type_engine, &decl_engine);
    let res = forc_check::check(command, engines)?;
    if !res.is_ok() {
        anyhow::bail!("unable to type check");
    }
    Ok(())
}
