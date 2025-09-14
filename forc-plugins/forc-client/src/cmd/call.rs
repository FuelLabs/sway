use crate::NodeTarget;
use clap::{Parser, ValueEnum};
use fuel_crypto::SecretKey;
use fuels::programs::calls::CallParameters;
use fuels_core::types::{Address, AssetId, ContractId};
use std::{io::Write, path::PathBuf, str::FromStr};
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
    /// JSON output with full tracing information (logs, errors, and result)
    Json,
}

impl Write for OutputFormat {
    fn write(&mut self, buf: &[u8]) -> Result<usize, std::io::Error> {
        match self {
            OutputFormat::Default => std::io::stdout().write(buf),
            OutputFormat::Raw => std::io::stdout().write(buf),
            OutputFormat::Json => Ok(buf.len()), // no-op for json
        }
    }

    fn flush(&mut self) -> Result<(), std::io::Error> {
        match self {
            OutputFormat::Default => std::io::stdout().flush(),
            OutputFormat::Raw => std::io::stdout().flush(),
            OutputFormat::Json => Ok(()),
        }
    }
}

impl From<OutputFormat> for forc_tracing::TracingWriter {
    fn from(format: OutputFormat) -> Self {
        match format {
            OutputFormat::Json => forc_tracing::TracingWriter::Json,
            _ => forc_tracing::TracingWriter::Stdio,
        }
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
#[derive(Debug, Clone, PartialEq)]
pub enum AbiSource {
    /// ABI from file path
    File(PathBuf),
    /// ABI from URL
    Url(Url),
    /// ABI as raw string
    String(String),
}

impl std::fmt::Display for AbiSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AbiSource::File(path) => write!(f, "{}", path.display()),
            AbiSource::Url(url) => write!(f, "{url}"),
            AbiSource::String(s) => write!(f, "{s}"),
        }
    }
}

impl TryFrom<String> for AbiSource {
    type Error = String;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        // First try to parse as URL
        if let Ok(url) = Url::parse(&s) {
            match url.scheme() {
                "http" | "https" | "ipfs" => return Ok(AbiSource::Url(url)),
                _ => {} // Continue to check other options
            }
        }

        // Check if it looks like a JSON string (starts with '{' or '[')
        let trimmed = s.trim();
        if (trimmed.starts_with('{') && trimmed.ends_with('}'))
            || (trimmed.starts_with('[') && trimmed.ends_with(']'))
        {
            return Ok(AbiSource::String(s));
        }

        // Default to treating as file path
        Ok(AbiSource::File(PathBuf::from(s)))
    }
}

#[derive(Debug, Clone)]
pub enum Operation {
    /// Call a specific contract function
    CallFunction {
        contract_id: ContractId,
        abi: AbiSource,
        function: FuncType,
        function_args: Vec<String>,
    },
    /// List all functions in the contract
    ListFunctions {
        contract_id: ContractId,
        abi: AbiSource,
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
#[clap(after_help = r#"
## EXAMPLES:

### Call a contract with function parameters
```sh
forc call 0x0dcba78d7b09a1f77353f51367afd8b8ab94b5b2bb6c9437d9ba9eea47dede97 \
    --abi ./contract-abi.json \
    get_balance 0x0087675439e10a8351b1d5e4cf9d0ea6da77675623ff6b16470b5e3c58998423
```

### Call a contract with function parameters; additionally print logs, receipts and script json
```sh
forc call 0x0dcba78d7b09a1f77353f51367afd8b8ab94b5b2bb6c9437d9ba9eea47dede97 \
    --abi ./contract-abi.json \
    get_balance 0x0087675439e10a8351b1d5e4cf9d0ea6da77675623ff6b16470b5e3c58998423 \
    -vv
```

### Call a contract with address labels for better trace readability
```sh
forc call 0x0dcba78d7b09a1f77353f51367afd8b8ab94b5b2bb6c9437d9ba9eea47dede97 \
    --abi ./contract-abi.json \
    transfer 0xf8f8b6283d7fa5b672b530cbb84fcccb4ff8dc40f8176ef4544ddb1f1952ad07 \
    --label 0x0dcba78d7b09a1f77353f51367afd8b8ab94b5b2bb6c9437d9ba9eea47dede97:MainContract \
    --label 0xf8f8b6283d7fa5b672b530cbb84fcccb4ff8dc40f8176ef4544ddb1f1952ad07:TokenContract \
    -vv
```

### Call a contract without function parameters
```sh
forc call 0x0dcba78d7b09a1f77353f51367afd8b8ab94b5b2bb6c9437d9ba9eea47dede97 \
    --abi ./contract-abi.json \
    get_name
```

### Call a contract that makes external contract calls
```sh
forc call 0x0dcba78d7b09a1f77353f51367afd8b8ab94b5b2bb6c9437d9ba9eea47dede97 \
    --abi ./contract-abi.json \
    transfer 0xf8f8b6283d7fa5b672b530cbb84fcccb4ff8dc40f8176ef4544ddb1f1952ad07 \
    --contracts 0xf8f8b6283d7fa5b672b530cbb84fcccb4ff8dc40f8176ef4544ddb1f1952ad07
```

### Call a contract with additional contract ABIs for better tracing
```sh
forc call 0x0dcba78d7b09a1f77353f51367afd8b8ab94b5b2bb6c9437d9ba9eea47dede97 \
    --abi ./contract-abi.json \
    transfer 0xf8f8b6283d7fa5b672b530cbb84fcccb4ff8dc40f8176ef4544ddb1f1952ad07 \
    --contract-abi 0xf8f8b6283d7fa5b672b530cbb84fcccb4ff8dc40f8176ef4544ddb1f1952ad07:./external-abi.json \
    --contract-abi 0x1234:https://example.com/abi.json
```

### Call a contract in simulation mode
```sh
forc call 0x0dcba78d7b09a1f77353f51367afd8b8ab94b5b2bb6c9437d9ba9eea47dede97 \
    --abi ./contract-abi.json \
    add 1 2 \
    --mode simulate
```

### Call a contract in dry-run mode on custom node URL using explicit signing-key
```sh
forc call 0x0dcba78d7b09a1f77353f51367afd8b8ab94b5b2bb6c9437d9ba9eea47dede97 \
    --node-url "http://127.0.0.1:4000/v1/graphql" \
    --signing-key 0x... \
    --abi ./contract-abi.json \
    add 1 2 \
    --mode dry-run
```

### Call a contract in live mode which performs state changes on testnet using forc-wallet
```sh
forc call 0x0dcba78d7b09a1f77353f51367afd8b8ab94b5b2bb6c9437d9ba9eea47dede97 \
    --testnet \
    --wallet \
    --abi ./contract-abi.json \
    add 1 2 \
    --mode live
```

### Call a contract payable function which transfers value of native asset on mainnet
```sh
forc call 0x0dcba78d7b09a1f77353f51367afd8b8ab94b5b2bb6c9437d9ba9eea47dede97 \
    --abi ./contract-abi.json \
    transfer 0xf8f8b6283d7fa5b672b530cbb84fcccb4ff8dc40f8176ef4544ddb1f1952ad07 \
    --mode live \
    --amount 100
```

### Call a contract payable function which transfers value of custom asset
```sh
forc call 0x0dcba78d7b09a1f77353f51367afd8b8ab94b5b2bb6c9437d9ba9eea47dede97 \
    --abi ./contract-abi.json \
    transfer 0xf8f8b6283d7fa5b672b530cbb84fcccb4ff8dc40f8176ef4544ddb1f1952ad07 \
    --amount 100 \
    --asset-id 0x0087675439e10a8351b1d5e4cf9d0ea6da77675623ff6b16470b5e3c58998423 \
    --live
```

### List all available functions in a contract
```sh
forc call 0x0dcba78d7b09a1f77353f51367afd8b8ab94b5b2bb6c9437d9ba9eea47dede97 \
    --abi ./contract-abi.json \
    --list-functions
```

### Call a contract with inline ABI JSON string
```sh
forc call 0x0dcba78d7b09a1f77353f51367afd8b8ab94b5b2bb6c9437d9ba9eea47dede97 \
    --abi '{"functions":[{"inputs":[],"name":"get_balance","output":{"name":"","type":"u64","typeArguments":null}}]}' \
    get_balance
```

### Direct transfer of asset to a contract or address
```sh
forc call 0x0dcba78d7b09a1f77353f51367afd8b8ab94b5b2bb6c9437d9ba9eea47dede97 \
    --amount 100 \
    --mode live
```

### Call a contract with interactive debugger after transaction
```sh
forc call 0x0dcba78d7b09a1f77353f51367afd8b8ab94b5b2bb6c9437d9ba9eea47dede97 \
    --abi ./contract-abi.json \
    get_balance 0x0087675439e10a8351b1d5e4cf9d0ea6da77675623ff6b16470b5e3c58998423 \
    --debug
```
"#)]
pub struct Command {
    /// The contract ID to call
    #[clap(help_heading = "CONTRACT")]
    pub address: Address,

    /// Path, URI, or raw JSON string for the ABI
    /// Required when making function calls or listing functions
    /// Can be a file path, HTTP/HTTPS URL, or raw JSON string
    #[clap(long, value_parser = |s: &str| AbiSource::try_from(s.to_string()))]
    pub abi: Option<AbiSource>,

    /// Additional contract IDs and their ABI paths for better tracing and debugging.
    /// Format: contract_id:abi_path (can be used multiple times)
    /// Example: --contract-abi 0x123:./abi1.json --contract-abi 0x456:https://example.com/abi2.json
    /// Contract IDs can be provided with or without 0x prefix
    #[clap(long = "contract-abi", value_parser = parse_contract_abi, action = clap::ArgAction::Append, help_heading = "CONTRACT")]
    pub contract_abis: Option<Vec<(ContractId, AbiSource)>>,

    /// Label addresses in the trace output for better readability.
    /// Format: address:label (can be used multiple times)
    /// Example: --label 0x123:MainContract --label 0x456:TokenContract
    /// Addresses can be provided with or without 0x prefix
    #[clap(long, value_parser = parse_label, action = clap::ArgAction::Append, help_heading = "OUTPUT")]
    pub label: Option<Vec<(ContractId, String)>>,

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
    /// If none are provided, the call will automatically populate external contracts by making dry-run calls
    /// to the node, and extract the contract addresses based on the revert reason.
    /// Use an empty string '' to explicitly specify no external contracts.
    /// Multiple contract IDs can be provided separated by commas.
    #[clap(
        long,
        alias = "contracts",
        value_delimiter = ',',
        help_heading = "CONTRACT"
    )]
    pub external_contracts: Option<Vec<String>>,

    /// Output format for the call result
    #[clap(long, short = 'o', default_value = "default", help_heading = "OUTPUT")]
    pub output: OutputFormat,

    /// Contract call variable output count
    #[clap(long, alias = "variable-output", help_heading = "VARIABLE OUTPUT")]
    pub variable_output: Option<usize>,

    /// Set verbosity levels; currently only supports max 2 levels
    /// - `-v=1`: Print decoded logs
    /// - `-v=2`: Additionally print receipts and script json
    #[clap(short = 'v', action = clap::ArgAction::Count, help_heading = "OUTPUT")]
    pub verbosity: u8,

    /// Start interactive debugger after transaction execution
    #[clap(long, short = 'd', help_heading = "DEBUG")]
    pub debug: bool,
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
                recipient: self.address,
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

fn parse_contract_abi(s: &str) -> Result<(ContractId, AbiSource), String> {
    let parts: Vec<&str> = s.trim().split(':').collect();
    let [contract_id_str, abi_path_str] = parts.try_into().map_err(|_| {
        format!("Invalid contract ABI format: '{s}'. Expected format: contract_id:abi_path")
    })?;

    let contract_id =
        ContractId::from_str(&format!("0x{}", contract_id_str.trim_start_matches("0x")))
            .map_err(|e| format!("Invalid contract ID '{contract_id_str}': {e}"))?;

    let abi_path = AbiSource::try_from(abi_path_str.to_string())
        .map_err(|e| format!("Invalid ABI path '{abi_path_str}': {e}"))?;

    Ok((contract_id, abi_path))
}

fn parse_label(s: &str) -> Result<(ContractId, String), String> {
    let parts: Vec<&str> = s.trim().split(':').collect();
    let [contract_id_str, label] = parts
        .try_into()
        .map_err(|_| format!("Invalid label format: '{s}'. Expected format: contract_id:label"))?;

    let contract_id =
        ContractId::from_str(&format!("0x{}", contract_id_str.trim_start_matches("0x")))
            .map_err(|e| format!("Invalid contract ID '{contract_id_str}': {e}"))?;

    Ok((contract_id, label.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_abi_source_try_from() {
        let url_result = AbiSource::try_from("https://example.com/abi.json".to_string()).unwrap();
        assert!(matches!(url_result, AbiSource::Url(_)));

        let json_result = AbiSource::try_from(r#"{"functions": []}"#.to_string()).unwrap();
        assert!(matches!(json_result, AbiSource::String(_)));

        let array_result = AbiSource::try_from("[]".to_string()).unwrap();
        assert!(matches!(array_result, AbiSource::String(_)));

        let file_result = AbiSource::try_from("./contract-abi.json".to_string()).unwrap();
        assert!(matches!(file_result, AbiSource::File(_)));

        let file_url_result = AbiSource::try_from("file:///path/to/abi.json".to_string()).unwrap();
        assert!(matches!(file_url_result, AbiSource::File(_)));
    }
}
