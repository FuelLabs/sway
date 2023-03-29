use clap::Parser;
use fuel_crypto::SecretKey;

pub use forc::cli::shared::{BuildOutput, BuildProfile, Minify, Pkg, Print};
pub use forc_tx::{Gas, Maturity, Salt};

#[derive(Debug, Default, Parser)]
#[clap(bin_name = "forc deploy", version)]
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
    /// Optional 256-bit hexadecimal literal to redeploy contracts.
    ///
    /// For a single contract, use `--salt <SALT>`.
    /// For a workspace with multiple contracts, use `--salt <CONTRACT_NAME>:<SALT>`
    /// to specify salts for multiple contracts.
    #[clap(long)]
    pub salt: Option<Vec<String>>,
    /// Generate a random salt for the contract.
    /// Useful for testing or deploying examples to a shared network.
    #[clap(long)]
    pub random_salt: bool,
    #[clap(flatten)]
    pub build_output: BuildOutput,
    #[clap(flatten)]
    pub build_profile: BuildProfile,
    /// The URL of the Fuel node to which we're submitting the transaction.
    /// If unspecified, checks the manifest's `network` table, then falls back
    /// to [`crate::default::NODE_URL`].
    #[clap(long, env = "FUEL_NODE_URL")]
    pub node_url: Option<String>,
    /// Do not sign the transaction
    #[clap(long)]
    pub unsigned: bool,
    /// Set the key to be used for signing.
    pub signing_key: Option<SecretKey>,
}
