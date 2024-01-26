use clap::Parser;
use fuel_crypto::SecretKey;

pub use forc::cli::shared::{BuildOutput, BuildProfile, Minify, Pkg, Print};
pub use forc_tx::{Gas, Maturity};
pub use forc_util::tx_utils::Salt;

use crate::NodeTarget;

forc_util::cli_examples! {
    [ Deploy a single contract => deploy "bc09bfa7a11a04ce42b0a5abf04fd437387ee49bf4561d575177e2946468b408" => r#".*Error making HTTP request.*"# ]
    [ Deploy a single contract from a different path => deploy "bc09bfa7a11a04ce42b0a5abf04fd437387ee49bf4561d575177e2946468b408 --path ../tests/" => r#".*Error making HTTP request.*"# ]
    [ Deploy to a custom network => deploy "--node-url https://beta-5.fuel.network/graphql" => ".*Deployment failed due to insufficient funds.*" ]
}

#[derive(Debug, Default, Parser)]
#[clap(bin_name = "forc deploy", version, after_help = help())]
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
    pub node: NodeTarget,
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
    /// Override storage slot initialization.
    ///
    /// By default, storage slots are initialized with the values defined in the storage block in
    /// the contract. You can override the initialization by providing the file path to a JSON file
    /// containing the overriden values.
    ///
    /// The file format and key values should match the compiler-generated `*-storage_slots.json` file in the output
    /// directory of the compiled contract.
    ///
    /// Example: `forc deploy --override-storage-slots my_override.json`
    ///
    /// my_override.json:
    /// [
    ///   {
    ///     "key": "<key from out/debug/storage_slots.json>",
    ///     "value": "0000000000000000000000000000000000000000000000000000000000000001"
    ///   }
    /// ]
    #[clap(long, verbatim_doc_comment, name = "JSON_FILE_PATH")]
    pub override_storage_slots: Option<String>,

    #[clap(long)]
    pub experimental_new_encoding: bool,
}
