use clap::Parser;
use fuel_gql_client::fuel_crypto::SecretKey;

pub use forc::cli::shared::{BuildOutput, BuildProfile, Minify, Pkg, Print};
pub use forc_tx::Gas;

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
    pub build_output: BuildOutput,
    #[clap(flatten)]
    pub build_profile: BuildProfile,
    /// The node url to deploy, if not specified uses DEFAULT_NODE_URL.
    /// If url is specified overrides network url in manifest file (if there is one).
    #[clap(long, env = "FUEL_NODE_URL")]
    pub node_url: Option<String>,
    /// Do not sign the transaction
    #[clap(long)]
    pub unsigned: bool,
    /// Set the key to be used for signing.
    pub signing_key: Option<SecretKey>,
}
