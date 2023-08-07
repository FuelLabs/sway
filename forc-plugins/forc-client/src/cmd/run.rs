use clap::Parser;
use fuel_crypto::SecretKey;

pub use super::submit::Network;
pub use forc::cli::shared::{BuildOutput, BuildProfile, Minify, Pkg, Print};
pub use forc_tx::{Gas, Maturity};

/// Run script project.
/// Crafts a script transaction then sends it to a running node.
#[derive(Debug, Default, Parser)]
#[clap(bin_name = "forc run", version)]
pub struct Command {
    #[clap(flatten)]
    pub pkg: Pkg,
    #[clap(flatten)]
    pub minify: Minify,
    #[clap(flatten)]
    pub print: Print,
    #[clap(flatten)]
    pub gas: Gas,
    #[clap(flatten)]
    pub maturity: Maturity,
    #[clap(flatten)]
    pub build_output: BuildOutput,
    #[clap(flatten)]
    pub build_profile: BuildProfile,
    /// The URL of the Fuel node to which we're submitting the transaction.
    /// If unspecified, checks the manifest's `network` table, then falls back
    /// to [`crate::default::NODE_URL`].
    #[clap(long, env = "FUEL_NODE_URL")]
    pub node_url: Option<String>,
    /// Hex string of data to input to script.
    #[clap(short, long)]
    pub data: Option<String>,
    /// Only craft transaction and print it out.
    #[clap(long)]
    pub dry_run: bool,
    /// Pretty-print the outputs from the node.
    #[clap(long = "pretty-print", short = 'r')]
    pub pretty_print: bool,
    /// 32-byte contract ID that will be called during the transaction.
    #[clap(long = "contract")]
    pub contract: Option<Vec<String>>,
    /// Execute the transaction and return the final mutated transaction along with receipts
    /// (which includes whether the transaction reverted or not). The transaction is not inserted
    /// in the node's view of the blockchain, (i.e. it does not affect the chain state).
    #[clap(long)]
    pub simulate: bool,
    /// Do not sign the transaction
    #[clap(long)]
    pub unsigned: bool,
    /// Set the key to be used for signing.
    pub signing_key: Option<SecretKey>,
    /// Sign the deployment transaction manually.
    #[clap(long)]
    pub manual_signing: bool,
    /// Arguments to pass into main function with forc run.
    #[clap(long)]
    pub args: Option<Vec<String>>,
}
