use crate::{
    cli::shared::{ManifestArgs, PackagesSelectionArgs, SectionArgs, SourceArgs},
    ops::forc_add,
};
use clap::Parser;
use forc_pkg::source::IPFSNode;
use forc_util::ForcResult;

forc_util::cli_examples! {
  crate::cli::Opt {
      [Add a dependencies => "forc add <DEP>[@<VERSION>] "]
      [Add a contract dependency => "forc add <DEP>[@<VERSION>] --contract-dep"]
      [Dry run  => "forc add <DEP>[@<VERSION>] --dry-run"]
  }
}

// Add dependencies to Forc toml
#[derive(Debug, Parser)]
#[clap(bin_name = "forc add", version, after_help = help())]
pub struct Command {
    /// List of dependencies to add in the format "name[@version]"
    #[clap(value_enum, value_name = "DEP_SPEC")]
    pub dependencies: Vec<String>,

    /// Print the changes that would be made without actually making them
    #[arg(long)]
    pub dry_run: bool,

    #[clap(flatten, next_help_heading = "Manifest Options")]
    pub manifest: ManifestArgs,

    #[clap(flatten, next_help_heading = "Package Selection")]
    pub package: PackagesSelectionArgs,

    #[clap(flatten, next_help_heading = "Source")]
    pub source: SourceArgs,

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
    match forc_add::add(command) {
        Ok(_) => Ok(()),
        Err(e) => Err(format!("couldn't add dependencies: {}", e).as_str().into()),
    }
}
