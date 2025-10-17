use clap::Parser;
use forc_util::ForcResult;
use sway_core::VerifyIr;

pub use crate::cli::shared::{BuildOutput, BuildProfile, Minify, Pkg, Print};
use crate::ops::forc_predicate_root;

forc_util::cli_examples! {
    crate::cli::Opt {
        [Get predicate root => "forc predicate-root"]
    }
}

/// Determine predicate-root for a predicate. For workspaces outputs all predicate roots in the
/// workspace.
#[derive(Debug, Parser)]
#[clap(bin_name = "forc predicate-root", version, after_help = help())]
pub struct Command {
    #[clap(flatten)]
    pub pkg: Pkg,
    #[clap(flatten)]
    pub minify: Minify,
    #[clap(flatten)]
    pub print: Print,
    #[clap(flatten)]
    pub verify_ir: VerifyIr,
    #[clap(flatten)]
    pub build_output: BuildOutput,
    #[clap(flatten)]
    pub build_profile: BuildProfile,

    #[clap(flatten)]
    pub experimental: sway_features::CliFields,
}

pub(crate) fn exec(cmd: Command) -> ForcResult<()> {
    forc_predicate_root::predicate_root(cmd).map_err(|e| e.into())
}
