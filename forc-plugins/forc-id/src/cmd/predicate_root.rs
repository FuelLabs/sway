use clap::Parser;

pub use forc::cli::shared::{BuildOutput, BuildProfile, Minify, Pkg, Print};

/// Determine the predicate-root for a predicate forc package, if run against a workspace outputs
/// all predicte roots in the workspace.
#[derive(Debug, Default, Parser)]
#[clap(bin_name = "forc predicate-root", version)]
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
