use crate::{
    cli::shared::{BuildOutput, BuildProfile, Minify, Pkg, Print},
    ops::forc_contract_id,
};
use anyhow::Result;
use clap::Parser;
use forc_util::Salt;

/// Determine contract-id for a contract. For workspaces outputs all contract ids in the workspace.
#[derive(Debug, Parser)]
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
}

pub(crate) fn exec(cmd: Command) -> Result<()> {
    forc_contract_id::contract_id(cmd)
}
