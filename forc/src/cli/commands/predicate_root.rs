use clap::Parser;
use forc_util::ForcResult;

pub use crate::cli::shared::{BuildOutput, BuildProfile, Minify, Pkg, Print};
use crate::ops::forc_predicate_root;

/// Determine predicate-root for a predicate. For workspaces outputs all predicate roots in the
/// workspace.
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
}

pub(crate) fn exec(cmd: Command) -> ForcResult<()> {
    forc_predicate_root::predicate_root(cmd).map_err(|e| e.into())
}
