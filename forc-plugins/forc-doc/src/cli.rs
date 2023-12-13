//! The command line interface for `forc doc`.
use clap::Parser;
use forc_pkg::source::IPFSNode;

const ABOUT: &str = "Forc plugin for building a Sway package's documentation";

const EXAMPLES: &str = r#"EXAMPLES:

    # Build the docs for the current project
    forc doc

    # Build the docs for the current project and open the browser
    forc doc --open

    # Build the docs for a project located in another path and open the browser
    forc doc --manifest-path /path/to/project --open

    # Build the docs for the current project, export private items/types and open the browser
    forc doc --document-private-items --open

    # Build the docs for the current project without downloading any new dependency through the network
    forc doc --offline
"#;

#[derive(Debug, Parser, Default)]
#[clap(
    name = "forc-doc",
    about = ABOUT,
    after_long_help = EXAMPLES,
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
