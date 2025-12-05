use crate::ops::forc_check;
use clap::Parser;
use forc_pkg::source::IPFSNode;
use forc_types::{forc_result_bail, ForcResult};
use sway_core::{BuildTarget, Engines};

forc_types::cli_examples! {
    crate::cli::Opt {
        [ Check the current project => "forc check" ]
        [ Check the current project with a different path => "forc check --path <PATH>" ]
        [ Check the current project without updating dependencies => "forc check --locked" ]
    }
}

/// Check the current or target project and all of its dependencies for errors.
///
/// This will essentially compile the packages without performing the final step of code generation,
/// which is faster than running forc build.
#[derive(Debug, Default, Parser)]
#[clap(bin_name = "forc check", version, after_help = help())]
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
    /// The IPFS Node to use for fetching IPFS sources.
    ///
    /// Possible values: FUEL, PUBLIC, LOCAL, <GATEWAY_URL>
    #[clap(long)]
    pub ipfs_node: Option<IPFSNode>,

    /// Dump all trait implementations for the given type name.
    #[clap(long = "dump-impls", value_name = "TYPE")]
    pub dump_impls: Option<String>,

    #[clap(flatten)]
    pub experimental: sway_features::CliFields,
}

pub(crate) fn exec(command: Command) -> ForcResult<()> {
    let engines = Engines::default();
    let res = forc_check::check(command, &engines)?;
    if res.0.is_none() {
        forc_result_bail!("unable to type check");
    }
    Ok(())
}
