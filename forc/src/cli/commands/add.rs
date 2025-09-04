use crate::cli::shared::{ManifestArgs, PackagesSelectionArgs, SectionArgs, SourceArgs};
use clap::Parser;
use forc_pkg::{
    manifest::dep_modifier::{self, Action, ModifyOpts},
    source::IPFSNode,
};
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
    #[clap(value_enum, value_name = "DEP_SPEC", required = true, num_args = 1..,)]
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
    dep_modifier::modify_dependencies(command.into())
        .map_err(|e| format!("failed to add dependencies: {e}"))
        .map_err(|msg| msg.as_str().into())
}

impl From<Command> for ModifyOpts {
    fn from(cmd: Command) -> Self {
        ModifyOpts {
            action: Action::Add,
            manifest_path: cmd.manifest.manisfest_path,
            package: cmd.package.package,
            source_path: cmd.source.path,
            git: cmd.source.git,
            branch: cmd.source.git_ref.branch,
            tag: cmd.source.git_ref.tag,
            rev: cmd.source.git_ref.rev,
            ipfs: cmd.source.ipfs,
            contract_deps: cmd.section.contract_deps,
            salt: cmd.section.salt,
            ipfs_node: cmd.ipfs_node,
            dependencies: cmd.dependencies,
            dry_run: cmd.dry_run,
            offline: cmd.offline,
        }
    }
}
