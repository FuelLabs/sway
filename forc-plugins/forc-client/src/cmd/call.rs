use crate::NodeTarget;
use clap::Parser;
use either::Either;
use fuel_crypto::SecretKey;
use fuels_core::types::ContractId;
use std::path::PathBuf;
use url::Url;

pub use forc::cli::shared::{BuildOutput, BuildProfile, Minify, Pkg, Print};
pub use forc_tx::{Gas, Maturity};

forc_util::cli_examples! {
    super::Command {
        [ Call a contract with function parameters => "forc call <CONTRACT_ID> <FUNCTION_SIGNATURE> <ARGS>" ]
        [ Call a contract without function parameters => "forc call <CONTRACT_ID> <FUNCTION_SIGNATURE>" ]
        [ Call a contract given an ABI file with function parameters => "forc call <CONTRACT_ID> --abi <ABI_FILE> <FUNCTION_SELECTOR> <ARGS>" ]
    }
}

#[derive(Debug, Clone)]
pub enum FuncType {
    Signature(String),
    Selector(String),
}

impl Default for FuncType {
    fn default() -> Self {
        FuncType::Signature(String::new())
    }
}

/// Call a contract function.
#[derive(Debug, Default, Parser)]
#[clap(bin_name = "forc call", version)]
pub struct Command {
    /// The contract ID to call.
    pub contract_id: ContractId,

    /// Optional path or URI to a JSON ABI file.
    #[clap(long, value_parser = parse_abi_path)]
    pub abi: Option<Either<PathBuf, Url>>,

    /// The function signature to call.
    /// When ABI is provided, this should be a selector (e.g. "transfer")
    /// When no ABI is provided, this should be the full function signature (e.g. "transfer(address,u64)")
    #[arg(value_parser = parse_signature_or_selector)]
    pub function: FuncType,

    /// Arguments to pass into main function with forc run.
    pub args: Vec<String>,

    #[clap(flatten)]
    pub gas: Option<Gas>,

    #[clap(flatten)]
    pub node: NodeTarget,

    /// Sign the transaction with default signer that is pre-funded by fuel-core. Useful for testing against local node.
    #[clap(long, default_value = "true")]
    pub default_signer: bool,

    /// Set the key to be used for signing; required if default_signer is false
    #[clap(long, required = false, required_if_eq("default_signer", "false"))]
    pub signing_key: Option<SecretKey>,
    // #[clap(flatten)]
    // pub experimental: sway_features::CliFields,
}

fn parse_abi_path(s: &str) -> Result<Either<PathBuf, Url>, String> {
    if let Ok(url) = Url::parse(s) {
        match url.scheme() {
            "http" | "https" | "ipfs" => Ok(Either::Right(url)),
            _ => Err(format!("Unsupported URL scheme: {}", url.scheme())),
        }
    } else {
        Ok(Either::Left(PathBuf::from(s)))
    }
}

fn parse_signature_or_selector(s: &str) -> Result<FuncType, String> {
    // remove all spaces
    let s = s.trim().replace(' ', "");
    if s.is_empty() {
        return Err("Function signature cannot be empty".to_string());
    }
    // Check if function signature is a valid selector (alphanumeric and underscore support)
    let selector_pattern = regex::Regex::new(r"^[a-zA-Z][a-zA-Z0-9_]*$").unwrap();
    if !selector_pattern.is_match(&s) {
        return Ok(FuncType::Signature(s.to_string()));
    }
    Ok(FuncType::Selector(s.to_string()))
}
