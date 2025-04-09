use crate::NodeTarget;
use clap::{Parser, ValueEnum};
use either::Either;
use fuel_crypto::SecretKey;
use fuels::programs::calls::CallParameters;
use fuels_core::types::{Address, AssetId, ContractId};
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

/// Execution mode for contract calls
#[derive(Debug, Clone, PartialEq, Default, ValueEnum)]
#[clap(rename_all = "kebab-case")]
pub enum ExecutionMode {
    /// Execute a dry run - no state changes, no gas fees, wallet is not used or validated
    #[default]
    DryRun,
    /// Execute in simulation mode - no state changes, estimates gas, wallet is used but not validated
    Simulate,
    /// Execute live on chain - state changes, gas fees apply, wallet is used and validated
    Live,
}

/// Output format for call results
#[derive(Debug, Clone, PartialEq, Default, ValueEnum)]
#[clap(rename_all = "lowercase")]
pub enum OutputFormat {
    /// Default formatted output
    #[default]
    Default,
    /// Raw unformatted output
    Raw,
}

/// Verbosity level for log output
#[derive(Debug, Clone, PartialEq, Default)]
#[repr(transparent)]
pub struct Verbosity(pub u8);

impl Verbosity {
    /// Verbose mode (-v)
    pub(crate) fn v1(&self) -> bool {
        self.0 >= 1
    }

    /// Very Verbose mode (-vv)
    pub(crate) fn v2(&self) -> bool {
        self.0 >= 2
    }
}

impl From<u8> for Verbosity {
    fn from(level: u8) -> Self {
        Verbosity(level)
    }
}

impl From<Verbosity> for u8 {
    fn from(verbosity: Verbosity) -> Self {
        verbosity.0
    }
}

/// Flags for specifying the caller account
#[derive(Debug, Default, Clone, Parser, serde::Deserialize, serde::Serialize)]
pub struct Caller {
    /// Derive an account from a secret key to make the call
    #[clap(long, env = "SIGNING_KEY", help_heading = "ACCOUNT OPTIONS")]
    pub signing_key: Option<SecretKey>,

    /// Use forc-wallet to make the call
    #[clap(long, default_value = "false", help_heading = "ACCOUNT OPTIONS")]
    pub wallet: bool,
}

/// Options for contract call parameters
#[derive(Debug, Default, Clone, Parser)]
pub struct CallParametersOpts {
    /// Amount of native assets to forward with the call
    #[clap(
        long,
        default_value = "0",
        alias = "value",
        help_heading = "CALL PARAMETERS"
    )]
    pub amount: u64,

    /// Asset ID to forward with the call
    #[clap(long, help_heading = "CALL PARAMETERS")]
    pub asset_id: Option<AssetId>,

    /// Amount of gas to forward with the call
    #[clap(long, help_heading = "CALL PARAMETERS")]
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

/// Operation for the call command
#[derive(Debug, Clone)]
pub enum Operation {
    /// Call a specific contract function
    CallFunction {
        contract_id: ContractId,
        abi: Either<PathBuf, Url>,
        function: FuncType,
        function_args: Vec<String>,
    },
    /// List all functions in the contract
    ListFunctions {
        contract_id: ContractId,
        abi: Either<PathBuf, Url>,
    },
    /// Direct transfer of assets to a contract
    DirectTransfer {
        recipient: Address,
        amount: u64,
        asset_id: Option<AssetId>,
    },
}

/// Perform Fuel RPC calls from the comfort of your command line.
#[derive(Debug, Parser, Clone)]
#[clap(bin_name = "forc call", version)]
#[clap(after_help = r#"EXAMPLES:

# Call a contract with function parameters
» forc call 0x0dcba78d7b09a1f77353f51367afd8b8ab94b5b2bb6c9437d9ba9eea47dede97 \
    --abi ./contract-abi.json \
    get_balance 0x0087675439e10a8351b1d5e4cf9d0ea6da77675623ff6b16470b5e3c58998423

# Call a contract with function parameters; additionally print logs, receipts and script json
» forc call 0x0dcba78d7b09a1f77353f51367afd8b8ab94b5b2bb6c9437d9ba9eea47dede97 \
    --abi ./contract-abi.json \
    get_balance 0x0087675439e10a8351b1d5e4cf9d0ea6da77675623ff6b16470b5e3c58998423 \
    -vv

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

# List all available functions in a contract
» forc call 0x0dcba78d7b09a1f77353f51367afd8b8ab94b5b2bb6c9437d9ba9eea47dede97 \
    --abi ./contract-abi.json \
    --list-functions

# Direct transfer of asset to a contract or address
» forc call 0x0dcba78d7b09a1f77353f51367afd8b8ab94b5b2bb6c9437d9ba9eea47dede97 \
    --amount 100 \
    --mode live
"#)]
pub struct Command {
    /// The contract ID to call
    #[clap(help_heading = "CONTRACT")]
    pub address: Address,

    /// Path or URI to a JSON ABI file
    /// Required when making function calls or listing functions
    #[clap(long, value_parser = parse_abi_path)]
    pub abi: Option<Either<PathBuf, Url>>,

    /// The function selector to call.
    /// The function selector is the name of the function to call (e.g. "transfer").
    /// Not required when --list-functions is specified or when --amount is provided for direct transfer
    #[clap(help_heading = "FUNCTION")]
    pub function: Option<String>,

    /// Arguments to pass to the function
    #[clap(help_heading = "FUNCTION")]
    pub function_args: Vec<String>,

    /// Network connection options
    #[clap(flatten)]
    pub node: NodeTarget,

    /// Caller account options
    #[clap(flatten)]
    pub caller: Caller,

    /// Call parameters
    #[clap(flatten)]
    pub call_parameters: CallParametersOpts,

    /// Execution mode - determines if state changes are applied
    /// - `dry-run`: No state changes, no gas fees, wallet is not used or validated
    /// - `simulate`: No state changes, estimates gas, wallet is used but not validated
    /// - `live`: State changes, gas fees apply, wallet is used and validated
    #[clap(long, default_value = "dry-run", help_heading = "EXECUTION")]
    pub mode: ExecutionMode,

    /// List all available functions in the contract
    #[clap(
        long,
        alias = "list-functions",
        conflicts_with = "function",
        help_heading = "OPERATION"
    )]
    pub list_functions: bool,

    /// The gas price to use for the call; defaults to 0
    #[clap(flatten)]
    pub gas: Option<Gas>,

    /// The external contract addresses to use for the call
    /// If none are provided, the call will automatically populate external contracts by making a dry-run calls
    /// to the node, and extract the contract addresses based on the revert reason
    #[clap(long, alias = "contracts", help_heading = "CONTRACT")]
    pub external_contracts: Option<Vec<ContractId>>,

    /// Output format for the call result
    #[clap(long, default_value = "default", help_heading = "OUTPUT")]
    pub output: OutputFormat,

    /// Set verbosity levels; currently only supports max 2 levels
    /// - `-v=1`: Print decoded logs
    /// - `-v=2`: Additionally print receipts and script json
    #[clap(short = 'v', action = clap::ArgAction::Count, help_heading = "OUTPUT")]
    pub verbosity: u8,
}

impl Command {
    /// Validate the command and determine the CLI operation
    pub fn validate_and_get_operation(&self) -> Result<Operation, String> {
        // Case 1: List functions
        if self.list_functions {
            let Some(abi) = &self.abi else {
                return Err("ABI is required when using --list-functions".to_string());
            };
            return Ok(Operation::ListFunctions {
                contract_id: (*self.address).into(),
                abi: abi.to_owned(),
            });
        }

        // Case 2: Direct transfer with amount
        if self.function.is_none() && self.call_parameters.amount > 0 {
            if self.mode != ExecutionMode::Live {
                return Err("Direct transfers are only supported in live mode".to_string());
            }
            return Ok(Operation::DirectTransfer {
                recipient: (*self.address).into(),
                amount: self.call_parameters.amount,
                asset_id: self.call_parameters.asset_id,
            });
        }

        // Case 3: Call function
        if let Some(function) = &self.function {
            let Some(abi) = &self.abi else {
                return Err("ABI is required when calling a function".to_string());
            };
            let func_type = FuncType::from_str(function)?;
            return Ok(Operation::CallFunction {
                contract_id: (*self.address).into(),
                abi: abi.to_owned(),
                function: func_type,
                function_args: self.function_args.to_owned(),
            });
        }

        // No valid operation matched
        Err("Either function selector, --list-functions flag, or non-zero --amount for direct transfers must be provided".to_string())
    }
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
