use clap::Parser;

pub use forc::cli::shared::{BuildOutput, BuildProfile, Minify, Pkg, Print};
use forc_tx::Salt;

/// Determine contract-id for a contract. For workspaces outputs all contract ids in the workspace.
#[derive(Debug, Default, Parser)]
#[clap(bin_name = "forc contract-id", version)]
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
