use crate::NodeTarget;
use clap::Parser;
use forc::cli::shared::PrintIrCliOpt;
pub use forc::cli::shared::{BuildOutput, Minify, Pkg, Print};
use forc_pkg::BuildProfile;
pub use forc_tx::{Gas, Maturity};
pub use forc_util::tx_utils::Salt;
use fuel_crypto::SecretKey;
use sway_core::IrCli;

forc_util::cli_examples! {
   super::Command {
        [ Deploy a single contract => "forc deploy bc09bfa7a11a04ce42b0a5abf04fd437387ee49bf4561d575177e2946468b408" ]
        [ Deploy a single contract from a different path => "forc deploy bc09bfa7a11a04ce42b0a5abf04fd437387ee49bf4561d575177e2946468b408 --path {path}" ]
        [ Deploy to a custom network => "forc deploy --node-url https://testnet.fuel.network/graphql" ]
    }
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
    #[arg(long, value_parser = clap::builder::PossibleValuesParser::new(PrintIrCliOpt::cli_options()))]
    pub verify_ir: IrCli,
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
    /// Useful for CI, to create reproducible deployments.
    #[clap(long)]
    pub default_salt: bool,
    #[clap(flatten)]
    pub build_output: BuildOutput,
    /// The name of the build profile to use.
    #[clap(long, default_value = BuildProfile::RELEASE)]
    pub build_profile: String,
    /// Sign the transaction with default signer that is pre-funded by fuel-core. Useful for testing against local node.
    #[clap(long)]
    pub default_signer: bool,
    /// Deprecated in favor of `--default-signer`.
    #[clap(long)]
    pub unsigned: bool,
    /// Submit the deployment transaction(s) without waiting for execution to complete.
    #[clap(long)]
    pub submit_only: bool,
    /// Set the key to be used for signing.
    pub signing_key: Option<SecretKey>,
    /// Sign the deployment transaction manually.
    #[clap(long)]
    pub manual_signing: bool,
    /// Override storage slot initialization.
    ///
    /// By default, storage slots are initialized with the values defined in the storage block in
    /// the contract. You can override the initialization by providing the file path to a JSON file
    /// containing the overridden values.
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

    #[clap(flatten)]
    pub experimental: sway_features::CliFields,

    /// AWS KMS signer arn. If present forc-deploy will automatically use AWS KMS signer instead of forc-wallet.
    #[clap(long)]
    pub aws_kms_signer: Option<String>,
}
