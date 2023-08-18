use clap::Parser;
use fuel_crypto::SecretKey;

pub use crate::util::Target;
pub use forc::cli::shared::{BuildOutput, BuildProfile, Minify, Pkg, Print};
pub use forc_tx::{Gas, Maturity};
pub use forc_util::tx_utils::Salt;

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
    /// Optional 256-bit hexadecimal literal(s) to redeploy contracts.
    ///
    /// For a single contract, use `--salt <SALT>`, eg.: forc deploy --salt 0x0000000000000000000000000000000000000000000000000000000000000001
    ///
    /// For a workspace with multiple contracts, use `--salt <CONTRACT_NAME>:<SALT>`
    /// to specify a salt for each contract, eg.:
    ///
    /// forc deploy --salt contract_a:0x0000000000000000000000000000000000000000000000000000000000000001
    /// --salt contract_b:0x0000000000000000000000000000000000000000000000000000000000000002
    #[clap(long)]
    pub salt: Option<Vec<String>>,
    /// Generate a default salt (0x0000000000000000000000000000000000000000000000000000000000000000) for the contract.
    /// Useful for CI, to create reproducable deployments.
    #[clap(long)]
    pub default_salt: bool,
    #[clap(flatten)]
    pub build_output: BuildOutput,
    #[clap(flatten)]
    pub build_profile: BuildProfile,
    /// The URL of the Fuel node to which we're submitting the transaction.
    /// If unspecified, checks the manifest's `network` table, then falls back
    /// to [`crate::default::NODE_URL`].
    #[clap(long, env = "FUEL_NODE_URL")]
    pub node_url: Option<String>,
    /// Sign the transaction with default signer that is pre-funded by fuel-core. Useful for testing against local node.
    #[clap(long)]
    pub default_signer: bool,
    /// Deprecated in favor of `--default-signer`.
    #[clap(long)]
    pub unsigned: bool,
    /// Set the key to be used for signing.
    pub signing_key: Option<SecretKey>,
    /// Sign the deployment transaction manually.
    #[clap(long)]
    pub manual_signing: bool,
    /// Use preset configurations for deploying to a specific target.
    ///
    /// Possible values are: [beta-1, beta-2, beta-3, beta-4, local]
    #[clap(long)]
    pub target: Option<Target>,
    /// Use preset configuration for the latest testnet.
    #[clap(long)]
    pub testnet: bool,
}
