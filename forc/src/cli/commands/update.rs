use crate::ops::forc_update;
use clap::Parser;
use forc_pkg::source::IPFSNode;
use forc_types::ForcResult;

forc_types::cli_examples! {
    crate::cli::Opt {
        [Update dependencies => "forc update"]
        [Update a specific dependency => "forc update -d std"]
        [Check if dependencies have newer versions => "forc update --check"]
    }
}

/// Update dependencies in the Forc dependencies directory.
#[derive(Debug, Default, Parser)]
#[clap(bin_name = "forc update", version, after_help = help())]
pub struct Command {
    /// Path to the project, if not specified, current working directory will be used.
    #[clap(short, long)]
    pub path: Option<String>,

    /// Dependency to be updated.
    /// If not set, all dependencies will be updated.
    #[clap(short = 'd')]
    pub target_dependency: Option<String>,

    /// Checks if the dependencies have newer versions.
    /// Won't actually perform the update, will output which
    /// ones are up-to-date and outdated.
    #[clap(short, long)]
    pub check: bool,

    /// The IPFS Node to use for fetching IPFS sources.
    ///
    /// Possible values: FUEL, PUBLIC, LOCAL, <GATEWAY_URL>
    #[clap(long)]
    pub ipfs_node: Option<IPFSNode>,
}

pub(crate) fn exec(command: Command) -> ForcResult<()> {
    match forc_update::update(command) {
        Ok(_) => Ok(()),
        Err(e) => Err(format!("couldn't update dependencies: {e}").as_str().into()),
    }
}
