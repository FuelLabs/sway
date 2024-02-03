use clap::Parser;
use forc_util::ForcResult;

pub use crate::cli::shared::{BuildOutput, BuildProfile, Minify, Pkg, Print};
use crate::ops::forc_predicate_root;

forc_util::cli_examples! {
    [Get predicate root => forc "predicate-root"]
    setup {
        crate::cli::create_project_and_compile(&forc_util::cli::get_cwd(), true);
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
    pub build_output: BuildOutput,
    #[clap(flatten)]
    pub build_profile: BuildProfile,

    #[clap(long)]
    /// Experimental flag for the "new encoding" feature
    pub experimental_new_encoding: bool,
}

pub(crate) fn exec(cmd: Command) -> ForcResult<()> {
    forc_predicate_root::predicate_root(cmd).map_err(|e| e.into())
}
