use crate::{
    cli::shared::{ManifestArgs, PackagesSelectionArgs, SectionArgs},
    ops::forc_remove,
};
use clap::Parser;
use forc_pkg::source::IPFSNode;
use forc_util::ForcResult;

forc_util::cli_examples! {
crate::cli::Opt {
    [Add a dependencies => "forc remove <DEP>"]
    [Add a contract dependency => "forc renove <DEP> --contract-dep"]
    [Dry run  => "forc remove <DEP> --dry-run"]
}
}

// Add dependencies to Forc toml
#[derive(Debug, Parser)]
#[clap(bin_name = "forc remove", version, after_help = help())]
pub struct Command {
    /// List of dependencies to remove in the format "name[@version]"
    #[clap(value_enum, value_name = "DEP_SPEC")]
    pub dependencies: Vec<String>,

    /// Print the changes that would be made without actually making them
    #[arg(long)]
    pub dry_run: bool,

    #[clap(flatten, next_help_heading = "Manifest Options")]
    pub manifest: ManifestArgs,

    #[clap(flatten, next_help_heading = "Package Selection")]
    pub package: PackagesSelectionArgs,

    #[clap(flatten, next_help_heading = "Section")]
    pub section: SectionArgs,

    /// Offline mode.
    ///
    /// Prevents Forc from using the network when managing dependencies.
    #[clap(long)]
    pub offline: bool,

    /// The IPFS Node to use for fetching IPFS sources.
    ///
    /// Possible values: FUEL, PUBLIC, LOCAL, <GATEWAY_URL>
    #[clap(long)]
    pub ipfs_node: Option<IPFSNode>,
}

pub(crate) fn exec(command: Command) -> ForcResult<()> {
    let _ = command;
    match forc_remove::remove(command) {
        Ok(_) => Ok(()),
        Err(e) => Err(format!("couldn't remove dependencies: {}", e)
            .as_str()
            .into()),
    }
}
