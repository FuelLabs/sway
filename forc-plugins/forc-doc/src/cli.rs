//! The command line interface for `forc doc`.
use clap::Parser;
use forc_pkg::source::IPFSNode;

const ABOUT: &str = "Forc plugin for building a Sway package's documentation";

forc::cli_examples! {
    [ Build the docs for a project in the current path => doc ""]
    [ Build the docs for a project in the current path and open it in the browser => doc "--open" ]
    [ Build the docs for a project located in another path => doc "--manifest-path ../tests_project2" ]
    [ Build the docs for the current project exporting private types => doc "--document-private-items" ]
    [ Build the docs offline without downloading any dependency from the network => doc "--offline" ]
}

#[derive(Debug, Parser, Default)]
#[clap(
    name = "forc-doc",
    about = ABOUT,
    after_help = help(),
    version
)]
pub struct Command {
    /// Path to the Forc.toml file. By default, forc-doc searches for the Forc.toml
    /// file in the current directory or any parent directory.
    #[clap(long)]
    pub manifest_path: Option<String>,
    /// Include non-public items in the documentation.
    #[clap(long)]
    pub document_private_items: bool,
    /// Open the docs in a browser after building them.
    #[clap(long)]
    pub open: bool,
    /// Offline mode, prevents Forc from using the network when managing dependencies.
    /// Meaning it will only try to use previously downloaded dependencies.
    #[clap(long = "offline")]
    pub offline: bool,
    /// Silent mode. Don't output any warnings or errors to the command line.
    #[clap(long = "silent", short = 's')]
    pub silent: bool,
    /// Requires that the Forc.lock file is up-to-date. If the lock file is missing, or it
    /// needs to be updated, Forc will exit with an error
    #[clap(long)]
    pub locked: bool,
    /// Do not build documentation for dependencies.
    #[clap(long)]
    pub no_deps: bool,
    /// The IPFS Node to use for fetching IPFS sources.
    ///
    /// Possible values: PUBLIC, LOCAL, <GATEWAY_URL>
    #[clap(long)]
    pub ipfs_node: Option<IPFSNode>,

    #[cfg(test)]
    pub(crate) doc_path: Option<String>,
}
