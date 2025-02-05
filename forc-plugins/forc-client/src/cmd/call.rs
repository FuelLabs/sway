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

#[derive(Debug, Clone)]
pub enum FuncType {
    Selector(String),
    // TODO: add support for function signatures - without ABI files
    // ↳ gh issue: https://github.com/FuelLabs/sway/issues/6886
    // Signature(String),
}

impl Default for FuncType {
    fn default() -> Self {
        FuncType::Selector(String::new())
    }
}

impl FromStr for FuncType {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim().replace(' ', "");
        if s.is_empty() {
            return Err("Function signature cannot be empty".to_string());
        }
        Ok(FuncType::Selector(s.to_string()))
    }
}

/// Flags for specifying the caller.
#[derive(Debug, Default, Clone, Parser, serde::Deserialize, serde::Serialize)]
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
    pub asset_id: Option<AssetId>,

    /// Amount of gas to forward with the call
    #[clap(long)]
    pub gas_forwarded: Option<u64>,
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

#[derive(Debug, Clone, PartialEq, Default)]
pub enum OutputFormat {
    #[default]
    Default,
    Raw,
}

impl FromStr for OutputFormat {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "default" => Ok(OutputFormat::Default),
            "raw" => Ok(OutputFormat::Raw),
            _ => Err(format!("Invalid output format: {}", s)),
        }
    }
}

/// Perform Fuel RPC calls from the comfort of your command line.
#[derive(Debug, Parser, Clone)]
#[clap(bin_name = "forc call", version)]
#[clap(after_help = r#"EXAMPLES:

# Call a contract with function parameters
» forc call 0x0dcba78d7b09a1f77353f51367afd8b8ab94b5b2bb6c9437d9ba9eea47dede97 \
    --abi ./contract-abi.json \
    get_balance 0x0087675439e10a8351b1d5e4cf9d0ea6da77675623ff6b16470b5e3c58998423

# Call a contract without function parameters
» forc call 0x0dcba78d7b09a1f77353f51367afd8b8ab94b5b2bb6c9437d9ba9eea47dede97 \
    --abi ./contract-abi.json \
    get_name

# Call a contract that makes external contract calls
» forc call 0x0dcba78d7b09a1f77353f51367afd8b8ab94b5b2bb6c9437d9ba9eea47dede97 \
    --abi ./contract-abi.json \
    transfer 0xf8f8b6283d7fa5b672b530cbb84fcccb4ff8dc40f8176ef4544ddb1f1952ad07 \
    --contracts 0xf8f8b6283d7fa5b672b530cbb84fcccb4ff8dc40f8176ef4544ddb1f1952ad07

# Call a contract in simulation mode
» forc call 0x0dcba78d7b09a1f77353f51367afd8b8ab94b5b2bb6c9437d9ba9eea47dede97 \
    --abi ./contract-abi.json \
    add 1 2 \
    --mode simulate

# Call a contract in dry-run mode on custom node URL using explicit signing-key
» forc call 0x0dcba78d7b09a1f77353f51367afd8b8ab94b5b2bb6c9437d9ba9eea47dede97 \
    --node-url "http://127.0.0.1:4000/v1/graphql" \
    --signing-key 0x... \
    --abi ./contract-abi.json \
    add 1 2 \
    --mode dry-run

# Call a contract in live mode which performs state changes on testnet using forc-wallet
» forc call 0x0dcba78d7b09a1f77353f51367afd8b8ab94b5b2bb6c9437d9ba9eea47dede97 \
    --testnet \
    --wallet \
    --abi ./contract-abi.json \
    add 1 2 \
    --mode live

# Call a contract payable function which transfers value of native asset on mainnet
» forc call 0x0dcba78d7b09a1f77353f51367afd8b8ab94b5b2bb6c9437d9ba9eea47dede97 \
    --abi ./contract-abi.json \
    transfer 0xf8f8b6283d7fa5b672b530cbb84fcccb4ff8dc40f8176ef4544ddb1f1952ad07 \
    --mode live \
    --amount 100

# Call a contract payable function which transfers value of custom asset
» forc call 0x0dcba78d7b09a1f77353f51367afd8b8ab94b5b2bb6c9437d9ba9eea47dede97 \
    --abi ./contract-abi.json \
    transfer 0xf8f8b6283d7fa5b672b530cbb84fcccb4ff8dc40f8176ef4544ddb1f1952ad07 \
    --amount 100 \
    --asset-id 0x0087675439e10a8351b1d5e4cf9d0ea6da77675623ff6b16470b5e3c58998423 \
    --live
"#)]
pub struct Command {
    /// The contract ID to call.
    pub contract_id: ContractId,

    /// Path or URI to a JSON ABI file.
    #[clap(long, value_parser = parse_abi_path)]
    pub abi: Either<PathBuf, Url>,

    /// The function selector to call.
    /// The function selector is the name of the function to call (e.g. "transfer").
    /// It must be a valid selector present in the ABI file.
    pub function: FuncType,

    /// Arguments to pass into the function to be called.
    pub function_args: Vec<String>,

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
    /// If none are provided, the call will automatically populate external contracts by making a dry-run calls
    /// to the node, and extract the contract addresses based on the revert reason
    #[clap(long, alias = "contracts")]
    pub external_contracts: Option<Vec<ContractId>>,

    /// The output format to use; possible values: default, raw
    #[clap(long, default_value = "default")]
    pub output: OutputFormat,
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
