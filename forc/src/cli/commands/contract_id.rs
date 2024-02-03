use crate::{
    cli::shared::{BuildOutput, BuildProfile, Minify, Pkg, Print},
    ops::forc_contract_id,
};
use clap::Parser;
use forc_util::{tx_utils::Salt, ForcResult};

forc_util::cli_examples! {
    [Get contract id => forc "contract-id"]
    [Get contract id from a different path => forc "contract-id --path /tmp/contract-id"]
    setup {
        crate::cli::create_project_and_compile("/tmp/contract-id", false);
    }
}

/// Determine contract-id for a contract. For workspaces outputs all contract
/// ids in the workspace.
#[derive(Debug, Parser)]
#[clap(bin_name = "forc contract-id", version, after_help = help())]
pub struct Command {
    #[clap(flatten)]
    pub pkg: Pkg,
    #[clap(flatten)]
    pub minify: Minify,
    #[clap(flatten)]
    pub print: Print,
    #[clap(flatten)]
    pub build_output: BuildOutput,
    #[clap(flatten)]
    pub build_profile: BuildProfile,
    #[clap(flatten)]
    pub salt: Salt,

    #[clap(long)]
    /// Experimental flag for the "new encoding" feature
    pub experimental_new_encoding: bool,
}

pub(crate) fn exec(cmd: Command) -> ForcResult<()> {
    forc_contract_id::contract_id(cmd).map_err(|e| e.into())
}
