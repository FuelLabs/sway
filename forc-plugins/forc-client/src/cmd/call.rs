use crate::NodeTarget;
use clap::Parser;
use either::Either;
use fuel_crypto::SecretKey;
use fuels::programs::calls::CallParameters;
use fuels_core::types::{AssetId, ContractId};
use std::{path::PathBuf, str::FromStr};
use url::Url;

pub use forc::cli::shared::{BuildOutput, BuildProfile, Minify, Pkg, Print};
pub use forc_tx::{Gas, Maturity};

forc_util::cli_examples! {
    super::Command {
        [ Call a contract with function parameters => "forc call <CONTRACT_ID> <FUNCTION_SIGNATURE> <ARGS>" ]
        [ Call a contract without function parameters => "forc call <CONTRACT_ID> <FUNCTION_SIGNATURE>" ]
        [ Call a contract given an ABI file with function parameters => "forc call <CONTRACT_ID> --abi <ABI_FILE> <FUNCTION_SELECTOR> <ARGS>" ]
        [ Call a contract that makes external contract calls => "forc call <CONTRACT_ID> --abi <ABI_FILE> <FUNCTION_SELECTOR> <ARGS> --contracts <CONTRACT_ADDRESS_1> <CONTRACT_ADDRESS_2>..." ]
        [ Call a contract in simulation mode => "forc call <CONTRACT_ID> <FUNCTION_SIGNATURE> --simulate" ]
        [ Call a contract in live mode which performs state changes => "forc call <CONTRACT_ID> <FUNCTION_SIGNATURE> --live" ]
        [ Call a contract payable function which transfers value of native asset => "forc call <CONTRACT_ID> <FUNCTION_SIGNATURE> --live --amount <VALUE>" ]
        [ Call a contract payable function which transfers value of custom asset => "forc call <CONTRACT_ID> <FUNCTION_SIGNATURE> --live --amount <VALUE> --asset-id <ASSET_ID>" ]
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

impl FromStr for FuncType {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
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
}

/// Flags for specifying the caller.
#[derive(Debug, Default, Parser, serde::Deserialize, serde::Serialize)]
pub struct Caller {
    /// Derive an account from a secret key to make the call
    #[clap(long, env = "SIGNING_KEY")]
    pub signing_key: Option<SecretKey>,

    /// Use forc-wallet to make the call
    #[clap(long, default_value = "false")]
    pub wallet: bool,
}

#[derive(Debug, Default, Clone, Parser)]
pub struct CallParametersOpts {
    /// Amount of native assets to forward with the call
    #[clap(long, default_value = "0", alias = "value")]
    pub amount: u64,

    /// Asset ID to forward with the call
    #[clap(long)]
    asset_id: Option<AssetId>,

    /// Amount of gas to forward with the call
    #[clap(long)]
    gas_forwarded: Option<u64>,
}

impl From<CallParametersOpts> for CallParameters {
    fn from(opts: CallParametersOpts) -> Self {
        let mut params = CallParameters::default();
        if opts.amount != 0 {
            params = params.with_amount(opts.amount);
        }
        if let Some(asset_id) = opts.asset_id {
            params = params.with_asset_id(asset_id);
        }
        if let Some(gas) = opts.gas_forwarded {
            params = params.with_gas_forwarded(gas);
        }
        params
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub enum ExecutionMode {
    /// Execute a dry run - no state changes, no gas fees, wallet is not used or validated
    #[default]
    DryRun,
    /// Execute in simulation mode - no state changes, estimates gas, wallet is used but not validated
    /// State changes are not applied
    Simulate,
    /// Execute live on chain - state changes, gas fees apply, wallet is used and validated
    /// State changes are applied
    Live,
}

impl FromStr for ExecutionMode {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "dry-run" => Ok(ExecutionMode::DryRun),
            "simulate" => Ok(ExecutionMode::Simulate),
            "live" => Ok(ExecutionMode::Live),
            _ => Err(format!("Invalid execution mode: {}", s)),
        }
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
    pub function: FuncType,

    /// Arguments to pass into main function with forc run.
    pub args: Vec<String>,

    #[clap(flatten)]
    pub node: NodeTarget,

    /// Select the caller to use for the call
    #[clap(flatten)]
    pub caller: Caller,

    /// Call parameters to use for the call
    #[clap(flatten)]
    pub call_parameters: CallParametersOpts,

    /// The execution mode to use for the call; defaults to dry-run; possible values: dry-run, simulate, live
    #[clap(long, default_value = "dry-run")]
    pub mode: ExecutionMode,

    /// The gas price to use for the call; defaults to 0
    #[clap(flatten)]
    pub gas: Option<Gas>,

    /// The external contract addresses to use for the call
    /// If none are provided, the call will automatically extract contract addresses from the function arguments
    /// and use them for the call as external contracts
    #[clap(long, alias = "contracts")]
    pub external_contracts: Option<Vec<ContractId>>,
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
