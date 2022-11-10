use clap::Parser;

#[derive(Debug, Parser)]
#[clap(
    name = "forc-doc",
    about = "Forc plugin for building a Sway package's documentation",
    version
)]
pub struct Command {
    /// Path to the Forc.toml file. By default, forc-doc searches for the Forc.toml
    /// file in the current directory or any parent directory.
    #[clap(long)]
    pub manifest_path: Option<String>,
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
}
