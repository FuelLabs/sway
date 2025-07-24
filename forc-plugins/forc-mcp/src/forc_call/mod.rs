mod resources;

use crate::McpToolModule;
use resources::{COMMON_COMMANDS_URI, CONTRACT_SAMPLES_URI, TYPE_ENCODING_REFERENCE_URI};
use rmcp::{
    handler::server::{router::tool::ToolRouter, tool::Parameters},
    model::*,
    schemars::{self, JsonSchema},
    service::RequestContext,
    tool, tool_handler, tool_router, Error as McpError, RoleServer, ServerHandler,
};
use serde::Deserialize;
use serde_json::Value;
use std::{collections::HashMap, future::Future, pin::Pin, str::FromStr};

#[derive(Debug, Deserialize, JsonSchema)]
pub struct CallContractArgs {
    pub contract_id: String,
    pub abi: String,
    pub function: String,
    #[serde(default)]
    pub function_args: Vec<String>,
    #[serde(default = "default_mode")]
    pub mode: String,
    pub node_url: Option<String>,
    pub signing_key: Option<String>,
    #[serde(default)]
    pub amount: u64,
    pub asset_id: Option<String>,
    pub gas_price: Option<u64>,
    #[serde(default)]
    pub verbosity: u8,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ListFunctionsArgs {
    pub contract_id: String,
    pub abi: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct TransferAssetsArgs {
    pub signing_key: String,
    pub recipient: String,
    pub amount: u64,
    pub asset_id: Option<String>,
    pub node_url: Option<String>,
    #[serde(default)]
    pub verbosity: u8,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetExecutionTraceArgs {
    /// JSON objects representing TraceEvent
    pub trace_events: Vec<HashMap<String, Value>>,
    /// Total gas used in the execution trace
    pub total_gas: u64,
    /// JSON string representation of HashMap<ContractId, String>
    pub labels: Option<HashMap<String, String>>,
}

fn default_mode() -> String {
    "dry-run".to_string()
}

/// Forc-call specific MCP tools
#[derive(Clone)]
pub struct ForcCallTools {
    pub tool_router: ToolRouter<ForcCallTools>,
}

#[tool_router]
impl ForcCallTools {
    pub fn new() -> Self {
        Self {
            tool_router: Self::tool_router(),
        }
    }

    #[tool(
        description = "Call a function on a deployed Fuel contract. Defaults to dry-run mode with default signer. Provide signing key to execute in live mode."
    )]
    async fn call_contract(
        &self,
        Parameters(args): Parameters<CallContractArgs>,
    ) -> Result<CallToolResult, McpError> {
        // Catch all errors and return them as error CallToolResults instead of McpErrors
        let cmd = match build_call_command(
            &args.contract_id,
            &args.abi,
            &args.function,
            args.function_args,
            &args.mode,
            args.node_url.as_deref(),
            args.signing_key.as_deref(),
            args.amount,
            args.asset_id.as_deref(),
            args.gas_price,
            args.verbosity,
        ) {
            Ok(cmd) => cmd,
            Err(e) => {
                return Ok(CallToolResult::error(vec![Content::text(format!(
                    "Error: Invalid arguments: {}",
                    e
                ))]))
            }
        };

        let operation = match cmd.validate_and_get_operation() {
            Ok(op) => op,
            Err(e) => {
                return Ok(CallToolResult::error(vec![Content::text(format!(
                    "Error: Failed to validate command: {}",
                    e
                ))]))
            }
        };

        let response = match forc_client::op::call(operation, cmd).await {
            Ok(resp) => resp,
            Err(e) => {
                return Ok(CallToolResult::error(vec![Content::text(format!(
                    "Error: Contract call failed: {}",
                    e
                ))]))
            }
        };

        let content = match Content::json(response) {
            Ok(content) => content,
            Err(e) => {
                return Ok(CallToolResult::error(vec![Content::text(format!(
                    "Failed to convert response to JSON: {}",
                    e
                ))]))
            }
        };

        Ok(CallToolResult::success(vec![content]))
    }

    #[tool(description = "List all callable functions in a contract's ABI with example usage.")]
    async fn list_contract_functions(
        &self,
        Parameters(args): Parameters<ListFunctionsArgs>,
    ) -> Result<CallToolResult, McpError> {
        let cmd = match build_list_command(&args.contract_id, &args.abi) {
            Ok(cmd) => cmd,
            Err(e) => {
                return Ok(CallToolResult::error(vec![Content::text(format!(
                    "Error: Invalid arguments: {}",
                    e
                ))]))
            }
        };

        let operation = match cmd.validate_and_get_operation() {
            Ok(op) => op,
            Err(e) => {
                return Ok(CallToolResult::error(vec![Content::text(format!(
                    "Error: Failed to validate command: {}",
                    e
                ))]))
            }
        };

        let (contract_id_parsed, abi_map) = match operation {
            forc_client::cmd::call::Operation::ListFunctions { contract_id, abi } => {
                match forc_client::op::call::create_abi_map(contract_id, &abi, cmd.contract_abis)
                    .await
                {
                    Ok(abi_map) => (contract_id, abi_map),
                    Err(e) => {
                        return Ok(CallToolResult::error(vec![Content::text(format!(
                            "Failed to create ABI map: {}",
                            e
                        ))]))
                    }
                }
            }
            _ => {
                return Ok(CallToolResult::error(vec![Content::text(
                    "Expected ListFunctions operation".to_string(),
                )]))
            }
        };

        let mut output_buffer = std::io::Cursor::new(Vec::<u8>::new());

        if let Err(e) = forc_client::op::call::list_functions::list_contract_functions(
            &contract_id_parsed,
            &abi_map,
            &mut output_buffer,
        ) {
            return Ok(CallToolResult::error(vec![Content::text(format!(
                "Failed to list contract functions: {}",
                e
            ))]));
        }

        let output_bytes = output_buffer.into_inner();
        let output_string = match String::from_utf8(output_bytes) {
            Ok(s) => s,
            Err(e) => {
                return Ok(CallToolResult::error(vec![Content::text(format!(
                    "Output was not valid UTF-8: {}",
                    e
                ))]))
            }
        };

        let content = Content::text(output_string);
        Ok(CallToolResult::success(vec![content]))
    }

    #[tool(
        description = "Transfer assets directly to an address or contract. Uses default signer and live mode."
    )]
    async fn transfer_assets(
        &self,
        Parameters(args): Parameters<TransferAssetsArgs>,
    ) -> Result<CallToolResult, McpError> {
        let cmd = match build_transfer_command(
            &args.signing_key,
            &args.recipient,
            args.amount,
            args.asset_id.as_deref(),
            args.node_url.as_deref(),
            args.verbosity,
        ) {
            Ok(cmd) => cmd,
            Err(e) => {
                return Ok(CallToolResult::error(vec![Content::text(format!(
                    "Error: Invalid arguments: {}",
                    e
                ))]))
            }
        };

        let operation = match cmd.validate_and_get_operation() {
            Ok(op) => op,
            Err(e) => {
                return Ok(CallToolResult::error(vec![Content::text(format!(
                    "Error: Failed to validate command: {}",
                    e
                ))]))
            }
        };

        let response = match forc_client::op::call(operation, cmd).await {
            Ok(resp) => resp,
            Err(e) => {
                return Ok(CallToolResult::error(vec![Content::text(format!(
                    "Error: Transfer failed: {}",
                    e
                ))]))
            }
        };

        let content = match Content::json(response) {
            Ok(content) => content,
            Err(e) => {
                return Ok(CallToolResult::error(vec![Content::text(format!(
                    "Failed to convert response to JSON: {}",
                    e
                ))]))
            }
        };

        Ok(CallToolResult::success(vec![content]))
    }

    #[tool(
        description = "Generate a formatted execution trace from trace events. Takes trace events from a CallResponse and returns a human-readable trace visualization."
    )]
    async fn get_execution_trace(
        &self,
        Parameters(args): Parameters<GetExecutionTraceArgs>,
    ) -> Result<CallToolResult, McpError> {
        // Parse trace events from array of JSON objects
        let mut trace_events: Vec<forc_client::op::call::trace::TraceEvent> = Vec::new();
        for event_obj in &args.trace_events {
            match serde_json::from_value(serde_json::Value::Object(
                event_obj.clone().into_iter().collect(),
            )) {
                Ok(event) => trace_events.push(event),
                Err(e) => {
                    return Ok(CallToolResult::error(vec![Content::text(format!(
                        "Error: Failed to parse trace_event: {}",
                        e
                    ))]))
                }
            }
        }

        // Convert labels from HashMap<String, String> to HashMap<ContractId, String>
        let labels: std::collections::HashMap<fuels_core::types::ContractId, String> =
            if let Some(labels_map) = args.labels {
                let mut converted_labels = std::collections::HashMap::new();
                for (contract_id_str, label) in labels_map {
                    match fuels_core::types::ContractId::from_str(&contract_id_str) {
                        Ok(contract_id) => {
                            converted_labels.insert(contract_id, label);
                        }
                        Err(e) => {
                            return Ok(CallToolResult::error(vec![Content::text(format!(
                                "Error: Failed to parse contract ID '{}': {}",
                                contract_id_str, e
                            ))]))
                        }
                    }
                }
                converted_labels
            } else {
                std::collections::HashMap::new()
            };

        // Create a buffer to capture the trace output
        let mut trace_buffer = Vec::new();

        // Generate the formatted trace
        if let Err(e) = forc_client::op::call::trace::display_transaction_trace(
            args.total_gas,
            &trace_events,
            &labels,
            &mut trace_buffer,
        ) {
            return Ok(CallToolResult::error(vec![Content::text(format!(
                "Error: Failed to generate trace: {}",
                e
            ))]));
        }

        let trace_output = match String::from_utf8(trace_buffer) {
            Ok(output) => output,
            Err(e) => {
                return Ok(CallToolResult::error(vec![Content::text(format!(
                    "Error: Failed to convert trace output to string: {}",
                    e
                ))]))
            }
        };

        Ok(CallToolResult::success(vec![Content::text(trace_output)]))
    }
}

impl Default for ForcCallTools {
    fn default() -> Self {
        Self::new()
    }
}

#[tool_handler]
impl ServerHandler for ForcCallTools {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: ProtocolVersion::V_2024_11_05,
            capabilities: ServerCapabilities::builder()
                .enable_tools()
                .enable_resources()
                .build(),
            server_info: Implementation {
                name: self.get_module_name().to_string(),
                version: env!("CARGO_PKG_VERSION").to_string(),
            },
            instructions: Some(
                "Forc-call specific MCP tools for contract interaction. Resources provide type encoding reference and examples.".to_string(),
            ),
        }
    }

    async fn list_resources(
        &self,
        _request: Option<PaginatedRequestParam>,
        _: RequestContext<RoleServer>,
    ) -> Result<ListResourcesResult, McpError> {
        Ok(ListResourcesResult {
            resources: vec![
                RawResource {
                    uri: TYPE_ENCODING_REFERENCE_URI.to_string(),
                    name: "MCP Type Encoding Reference".to_string(),
                    description: Some(
                        "Complete reference for encoding Sway types as MCP tool parameters"
                            .to_string(),
                    ),
                    mime_type: Some("text/markdown".to_string()),
                    size: None,
                }
                .no_annotation(),
                RawResource {
                    uri: COMMON_COMMANDS_URI.to_string(),
                    name: "MCP Tool Usage Examples".to_string(),
                    description: Some(
                        "Examples of common MCP tool usage patterns and parameters".to_string(),
                    ),
                    mime_type: Some("text/markdown".to_string()),
                    size: None,
                }
                .no_annotation(),
                RawResource {
                    uri: CONTRACT_SAMPLES_URI.to_string(),
                    name: "Contract Examples with MCP Tools".to_string(),
                    description: Some(
                        "Sample Sway contracts with MCP tool usage examples".to_string(),
                    ),
                    mime_type: Some("text/markdown".to_string()),
                    size: None,
                }
                .no_annotation(),
            ],
            next_cursor: None,
        })
    }

    async fn read_resource(
        &self,
        ReadResourceRequestParam { uri }: ReadResourceRequestParam,
        ctx: RequestContext<RoleServer>,
    ) -> Result<ReadResourceResult, McpError> {
        resources::read_resource(&uri, ctx).await
    }
}

impl McpToolModule for ForcCallTools {
    fn get_module_name(&self) -> &'static str {
        "forc-call-tools"
    }

    fn list_tools(
        &self,
        request: Option<PaginatedRequestParam>,
        ctx: RequestContext<RoleServer>,
    ) -> Pin<Box<dyn Future<Output = Result<ListToolsResult, McpError>> + Send>> {
        let self_clone = self.clone();
        Box::pin(async move { ServerHandler::list_tools(&self_clone, request, ctx).await })
    }

    fn call_tool(
        &self,
        request: CallToolRequestParam,
        ctx: RequestContext<RoleServer>,
    ) -> Pin<Box<dyn Future<Output = Result<CallToolResult, McpError>> + Send>> {
        let self_clone = self.clone();
        Box::pin(async move { ServerHandler::call_tool(&self_clone, request, ctx).await })
    }

    fn list_resources(
        &self,
        request: Option<PaginatedRequestParam>,
        ctx: RequestContext<RoleServer>,
    ) -> Pin<Box<dyn Future<Output = Result<ListResourcesResult, McpError>> + Send>> {
        let self_clone = self.clone();
        Box::pin(async move { ServerHandler::list_resources(&self_clone, request, ctx).await })
    }

    fn read_resource(
        &self,
        request: ReadResourceRequestParam,
        ctx: RequestContext<RoleServer>,
    ) -> Pin<Box<dyn Future<Output = Result<ReadResourceResult, McpError>> + Send>> {
        let self_clone = self.clone();
        Box::pin(async move { ServerHandler::read_resource(&self_clone, request, ctx).await })
    }

    fn get_info(&self) -> ServerInfo {
        ServerHandler::get_info(self)
    }
}

// Helper functions for building forc-client commands
#[allow(clippy::too_many_arguments)]
fn build_call_command(
    contract_id: &str,
    abi: &str,
    function: &str,
    function_args: Vec<String>,
    mode: &str,
    node_url: Option<&str>,
    signing_key: Option<&str>,
    amount: u64,
    asset_id: Option<&str>,
    gas_price: Option<u64>,
    verbosity: u8,
) -> anyhow::Result<forc_client::cmd::Call> {
    use forc_client::cmd::call::*;
    use fuels_core::types::{Address, AssetId};
    use std::str::FromStr;

    let address = Address::from_str(contract_id)
        .map_err(|e| anyhow::anyhow!("Invalid contract address: {}", e))?;

    let abi_source =
        AbiSource::try_from(abi.to_string()).map_err(|e| anyhow::anyhow!("Invalid ABI: {}", e))?;

    let execution_mode = match mode {
        "dry-run" => ExecutionMode::DryRun,
        "simulate" => ExecutionMode::Simulate,
        "live" => ExecutionMode::Live,
        _ => {
            return Err(anyhow::anyhow!(
                "Invalid mode. Use: dry-run, simulate, or live"
            ))
        }
    };

    let signing_key_parsed = if let Some(key) = signing_key {
        Some(
            fuel_crypto::SecretKey::from_str(key)
                .map_err(|e| anyhow::anyhow!("Invalid signing key: {}", e))?,
        )
    } else {
        None
    };

    let asset_id_parsed = if let Some(id) = asset_id {
        Some(AssetId::from_str(id).map_err(|e| anyhow::anyhow!("Invalid asset ID: {}", e))?)
    } else {
        None
    };

    let node = forc_client::NodeTarget {
        node_url: node_url.map(String::from),
        target: None,
        testnet: false,
        mainnet: false,
        devnet: false,
    };

    let gas = gas_price.map(|price| forc_tx::Gas {
        price: Some(price),
        script_gas_limit: None,
        max_fee: None,
        tip: None,
    });

    Ok(forc_client::cmd::Call {
        address,
        abi: Some(abi_source),
        contract_abis: None,
        label: None,
        function: Some(function.to_string()),
        function_args,
        node,
        caller: Caller {
            signing_key: signing_key_parsed,
            wallet: false,
        },
        call_parameters: CallParametersOpts {
            amount,
            asset_id: asset_id_parsed,
            gas_forwarded: None,
        },
        mode: execution_mode,
        list_functions: false,
        gas,
        external_contracts: None,
        output: OutputFormat::Json,
        verbosity,
    })
}

fn build_list_command(contract_id: &str, abi: &str) -> anyhow::Result<forc_client::cmd::Call> {
    use forc_client::cmd::call::*;
    use fuels_core::types::Address;
    use std::str::FromStr;

    let address = Address::from_str(contract_id)
        .map_err(|e| anyhow::anyhow!("Invalid contract address: {}", e))?;

    let abi_source =
        AbiSource::try_from(abi.to_string()).map_err(|e| anyhow::anyhow!("Invalid ABI: {}", e))?;

    let node = forc_client::NodeTarget {
        node_url: None,
        target: None,
        testnet: false,
        mainnet: false,
        devnet: false,
    };

    Ok(forc_client::cmd::Call {
        address,
        abi: Some(abi_source),
        contract_abis: None,
        label: None,
        function: None,
        function_args: vec![],
        node,
        caller: Caller {
            signing_key: None,
            wallet: false,
        },
        call_parameters: CallParametersOpts::default(),
        mode: ExecutionMode::DryRun,
        list_functions: true,
        gas: None,
        external_contracts: None,
        output: OutputFormat::Default,
        verbosity: 0,
    })
}

fn build_transfer_command(
    signing_key: &str,
    recipient: &str,
    amount: u64,
    asset_id: Option<&str>,
    node_url: Option<&str>,
    verbosity: u8,
) -> anyhow::Result<forc_client::cmd::Call> {
    use forc_client::cmd::call::*;
    use fuels_core::types::{Address, AssetId};
    use std::str::FromStr;

    let signing_key_parsed = fuel_crypto::SecretKey::from_str(signing_key)
        .map_err(|e| anyhow::anyhow!("Invalid signing key: {}", e))?;

    let address = Address::from_str(recipient)
        .map_err(|e| anyhow::anyhow!("Invalid recipient address: {}", e))?;

    let asset_id_parsed = if let Some(id) = asset_id {
        Some(AssetId::from_str(id).map_err(|e| anyhow::anyhow!("Invalid asset ID: {}", e))?)
    } else {
        None
    };

    let node = forc_client::NodeTarget {
        node_url: node_url.map(String::from),
        target: None,
        testnet: false,
        mainnet: false,
        devnet: false,
    };

    Ok(forc_client::cmd::Call {
        address,
        abi: None,
        contract_abis: None,
        label: None,
        function: None,
        function_args: vec![],
        node,
        caller: Caller {
            signing_key: Some(signing_key_parsed),
            wallet: false,
        },
        call_parameters: CallParametersOpts {
            amount,
            asset_id: asset_id_parsed,
            gas_forwarded: None,
        },
        mode: ExecutionMode::Live,
        list_functions: false,
        gas: None,
        external_contracts: None,
        output: OutputFormat::Json,
        verbosity,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::ForcMcpClient;
    use anyhow::Result;
    use forc_client::cmd::call::{ExecutionMode, OutputFormat};
    use fuels::crypto::SecretKey;
    use fuels::prelude::*;
    use fuels_accounts::signers::private_key::PrivateKeySigner;
    use serde_json::Value;
    use std::{collections::HashMap, str::FromStr};

    #[test]
    fn test_call_contract_command_construction() {
        let cmd = build_call_command(
            "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef",
            "./test.json",
            "test_function",
            vec!["arg1".to_string(), "arg2".to_string()],
            "simulate",
            None,
            None,
            100,
            None,
            None,
            2,
        )
        .unwrap();

        assert_eq!(cmd.function.unwrap(), "test_function");
        assert_eq!(cmd.function_args, vec!["arg1", "arg2"]);
        assert_eq!(cmd.mode, ExecutionMode::Simulate);
        assert_eq!(cmd.call_parameters.amount, 100);
        assert_eq!(cmd.verbosity, 2);
        assert!(!cmd.list_functions);
        assert_eq!(cmd.output, OutputFormat::Json);
        assert!(cmd.abi.is_some());
        assert!(!cmd.caller.wallet);
    }

    #[test]
    fn test_list_functions_command_construction() {
        let cmd = build_list_command(
            "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef",
            "https://example.com/abi.json",
        )
        .unwrap();

        assert!(cmd.list_functions);
        assert_eq!(cmd.function, None);
        assert_eq!(cmd.function_args, Vec::<String>::new());
        assert_eq!(cmd.mode, ExecutionMode::DryRun);
        assert_eq!(cmd.output, OutputFormat::Default);
        assert_eq!(cmd.verbosity, 0);
        assert!(cmd.abi.is_some());
    }

    #[test]
    fn test_transfer_assets_command_construction() {
        let cmd = build_transfer_command(
            "0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890",
            "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef",
            500,
            None,
            None,
            1,
        )
        .unwrap();

        assert_eq!(cmd.mode, ExecutionMode::Live);
        assert_eq!(cmd.call_parameters.amount, 500);
        assert_eq!(cmd.verbosity, 1);
        assert_eq!(cmd.abi, None);
        assert_eq!(cmd.function, None);
        assert_eq!(cmd.function_args, Vec::<String>::new());
        assert!(!cmd.list_functions);
        assert_eq!(cmd.output, OutputFormat::Json);
    }

    #[test]
    fn test_forc_call_tools_available() {
        let tools = ForcCallTools::new();
        let tool_list = tools.tool_router.list_all();
        let tool_names: Vec<String> = tool_list
            .into_iter()
            .map(|tool| tool.name.to_string())
            .collect();

        assert_eq!(tool_names.len(), 4, "Should have exactly 4 forc-call tools");
        assert!(tool_names.contains(&"call_contract".to_string()));
        assert!(tool_names.contains(&"list_contract_functions".to_string()));
        assert!(tool_names.contains(&"transfer_assets".to_string()));
        assert!(tool_names.contains(&"get_execution_trace".to_string()));
    }

    struct E2ETestFixture {
        pub contract_id: String,
        pub abi_path: String,
        pub node_url: String,
        pub secret_key: String,
        pub provider: Provider,
    }

    impl E2ETestFixture {
        pub async fn new() -> Result<E2ETestFixture> {
            // Setup local node and deploy contract
            let secret_key = SecretKey::random(&mut rand::thread_rng());
            let signer = PrivateKeySigner::new(secret_key);

            let coins =
                setup_single_asset_coins(signer.address(), AssetId::zeroed(), 10, 1_000_000_000);
            let provider = setup_test_provider(coins, vec![], None, None).await?;

            let wallet = Wallet::new(signer, provider.clone());

            // Deploy the test contract
            let contract_id = Contract::load_from(
                "../../forc-plugins/forc-client/test/data/contract_with_types/contract_with_types.bin",
                LoadConfiguration::default(),
            )?
            .deploy(&wallet, TxPolicies::default())
            .await?
            .contract_id;

            // Use the existing ABI file directly (no temp file needed)
            let abi_path = "../../forc-plugins/forc-client/test/data/contract_with_types/contract_with_types-abi.json";

            Ok(E2ETestFixture {
                contract_id: format!("0x{}", contract_id),
                abi_path: abi_path.to_string(),
                node_url: provider.url().to_string(),
                secret_key: format!("0x{}", secret_key),
                provider,
            })
        }

        /// Helper to extract text from MCP Content - reusable across different MCP tools
        pub fn extract_text_from_content(content: &rmcp::model::Content) -> Option<String> {
            // Since we can't pattern match due to type constraints, we'll use serialization
            // This is a workaround for the complex generic type structure
            if let Ok(json) = serde_json::to_value(content) {
                if let Some(text) = json.get("text") {
                    if let Some(text_str) = text.as_str() {
                        return Some(text_str.to_string());
                    }
                }
            }
            None
        }

        /// Create arguments for contract call tool
        pub fn create_call_tool_args(
            &self,
            function: &str,
            function_args: Vec<&str>,
        ) -> HashMap<String, Value> {
            let mut args = HashMap::new();
            args.insert(
                "contract_id".to_string(),
                Value::String(self.contract_id.clone()),
            );
            args.insert("abi".to_string(), Value::String(self.abi_path.clone()));
            args.insert("function".to_string(), Value::String(function.to_string()));
            args.insert(
                "function_args".to_string(),
                Value::Array(
                    function_args
                        .into_iter()
                        .map(|s| Value::String(s.to_string()))
                        .collect(),
                ),
            );
            args.insert("node_url".to_string(), Value::String(self.node_url.clone()));
            args.insert(
                "signing_key".to_string(),
                Value::String(self.secret_key.clone()),
            );
            args.insert("mode".to_string(), Value::String("dry-run".to_string()));
            args
        }

        /// Create arguments for list functions tool
        pub fn create_list_tool_args(&self) -> HashMap<String, Value> {
            let mut args = HashMap::new();
            args.insert(
                "contract_id".to_string(),
                Value::String(self.contract_id.clone()),
            );
            args.insert("abi".to_string(), Value::String(self.abi_path.clone()));
            args.insert("node_url".to_string(), Value::String(self.node_url.clone()));
            args
        }

        /// Create arguments for transfer assets tool
        pub fn create_transfer_tool_args(
            &self,
            recipient: &str,
            amount: u64,
        ) -> HashMap<String, Value> {
            let mut args = HashMap::new();
            args.insert(
                "recipient".to_string(),
                Value::String(recipient.to_string()),
            );
            args.insert("amount".to_string(), Value::Number(amount.into()));
            args.insert("node_url".to_string(), Value::String(self.node_url.clone()));
            args.insert(
                "signing_key".to_string(),
                Value::String(self.secret_key.clone()),
            );
            args
        }

        /// Create arguments for get_execution_trace tool
        #[allow(dead_code)]
        pub fn create_trace_tool_args(
            &self,
            trace_events: &[forc_client::op::call::trace::TraceEvent],
            total_gas: u64,
            labels: Option<&std::collections::HashMap<fuels_core::types::ContractId, String>>,
        ) -> HashMap<String, Value> {
            let mut args = HashMap::new();

            // Convert each trace event to JSON object (HashMap<String, Value>)
            let trace_events_array: Vec<Value> = trace_events
                .iter()
                .map(|event| serde_json::to_value(event).unwrap())
                .collect();
            args.insert("trace_events".to_string(), Value::Array(trace_events_array));
            args.insert("total_gas".to_string(), Value::Number(total_gas.into()));

            if let Some(labels) = labels {
                // Convert HashMap<ContractId, String> to HashMap<String, String>
                let labels_map: HashMap<String, String> = labels
                    .iter()
                    .map(|(contract_id, label)| (format!("0x{}", contract_id), label.clone()))
                    .collect();
                args.insert(
                    "labels".to_string(),
                    serde_json::to_value(labels_map).unwrap(),
                );
            }

            args
        }
    }

    #[tokio::test]
    async fn test_forc_call_mcp_tools_available_via_http_mcp() -> Result<()> {
        // Test that all expected forc-call tools are available via the SSE server
        let mut client = ForcMcpClient::http_stream_client().await?;

        let tool_names = client.list_tools().await?;

        assert_eq!(tool_names.len(), 4, "Should have exactly 4 forc-call tools");
        assert!(
            tool_names.contains(&"call_contract".to_string()),
            "Should have call_contract tool"
        );
        assert!(
            tool_names.contains(&"list_contract_functions".to_string()),
            "Should have list_contract_functions tool"
        );
        assert!(
            tool_names.contains(&"transfer_assets".to_string()),
            "Should have transfer_assets tool"
        );
        assert!(
            tool_names.contains(&"get_execution_trace".to_string()),
            "Should have get_execution_trace tool"
        );

        Ok(())
    }

    #[tokio::test]
    async fn test_call_contract_tool_http_mcp() -> Result<()> {
        let fixture = E2ETestFixture::new().await.unwrap();
        let mut client = ForcMcpClient::http_stream_client().await?;

        // Test calling a simple function through the MCP SSE server
        let args = fixture.create_call_tool_args("test_u8", vec!["255"]);

        let result = client.call_tool("call_contract", args).await?;

        // Full validation for the first e2e test
        assert_eq!(result.is_error, Some(false), "Call should not be an error");
        assert!(!result.content.is_empty(), "Content should not be empty");

        // Extract and parse the response content
        let text = E2ETestFixture::extract_text_from_content(&result.content[0])
            .expect("Response content should be text");
        let call_response: serde_json::Value = serde_json::from_str(&text)?;

        // Verify the function returned the expected value
        assert!(
            call_response.get("tx_hash").is_some(),
            "Response should have tx_hash"
        );
        assert!(
            call_response.get("result").is_some(),
            "Response should have result"
        );
        assert_eq!(
            call_response["result"], "255",
            "Function should return input value"
        );

        Ok(())
    }

    #[tokio::test]
    async fn test_call_contract_with_complex_types_http_mcp() -> Result<()> {
        let fixture = E2ETestFixture::new().await.unwrap();
        let mut client = ForcMcpClient::http_stream_client().await?;

        // Test calling a function with complex parameter (tuple)
        let args = fixture.create_call_tool_args("test_tuple", vec!["(42, true)"]);

        let result = client.call_tool("call_contract", args).await?;

        assert!(!result.content.is_empty(), "Content should not be empty");

        let text = E2ETestFixture::extract_text_from_content(&result.content[0])
            .expect("Response content should be text");
        let call_response: serde_json::Value = serde_json::from_str(&text)?;

        assert_eq!(
            call_response["result"], "(42, true)",
            "Function should return tuple"
        );

        Ok(())
    }

    #[tokio::test]
    async fn test_call_contract_simulate_mode_http_mcp() -> Result<()> {
        let fixture = E2ETestFixture::new().await.unwrap();
        let mut client = ForcMcpClient::http_stream_client().await?;

        // Test calling in simulate mode
        let mut args = fixture.create_call_tool_args("test_empty", vec![]);
        args.insert(
            "mode".to_string(),
            serde_json::Value::String("simulate".to_string()),
        );

        let result = client.call_tool("call_contract", args).await?;

        assert!(!result.content.is_empty(), "Content should not be empty");

        let text = E2ETestFixture::extract_text_from_content(&result.content[0])
            .expect("Response content should be text");

        // In simulate mode, the call may fail with signature validation errors
        // Check if the response is an error or a valid result
        if text.starts_with("Error:") {
            // This is expected in simulate mode without proper setup
            assert!(
                text.contains("InputInvalidSignature"),
                "Expected signature validation error"
            );
        } else {
            // If it's not an error, it should be valid JSON
            let call_response: serde_json::Value = serde_json::from_str(&text)?;
            assert_eq!(call_response["result"], "()");
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_list_contract_functions_tool_http_mcp() -> Result<()> {
        let fixture = E2ETestFixture::new().await.unwrap();
        let mut client = ForcMcpClient::http_stream_client().await?;

        let args = fixture.create_list_tool_args();

        let result = client.call_tool("list_contract_functions", args).await?;

        assert!(!result.content.is_empty(), "Content should not be empty");

        let text = E2ETestFixture::extract_text_from_content(&result.content[0])
            .expect("Response content should be text");

        // The list functions operation returns the actual function listing
        assert!(
            text.contains("Callable functions for contract:"),
            "Response should contain function listing"
        );
        assert!(
            text.contains("forc call"),
            "Response should contain forc call examples"
        );

        Ok(())
    }

    #[tokio::test]
    async fn test_transfer_assets_tool_http_mcp() -> Result<()> {
        let fixture = E2ETestFixture::new().await.unwrap();
        let mut client = ForcMcpClient::http_stream_client().await?;

        // Create a random recipient
        let random_wallet = Wallet::random(&mut rand::thread_rng(), fixture.provider.clone());
        let recipient_address = format!("0x{}", random_wallet.address());

        // Get initial balance
        let consensus_parameters = fixture.provider.consensus_parameters().await?;
        let base_asset_id = consensus_parameters.base_asset_id();
        let initial_balance = fixture
            .provider
            .get_asset_balance(&random_wallet.address(), base_asset_id)
            .await?;

        // Test transferring assets through MCP SSE server
        let transfer_amount = 1000u64;
        let args = fixture.create_transfer_tool_args(&recipient_address, transfer_amount);

        let result = client.call_tool("transfer_assets", args).await?;

        assert!(!result.content.is_empty(), "Content should not be empty");

        let text = E2ETestFixture::extract_text_from_content(&result.content[0])
            .expect("Response content should be text");
        let transfer_response: serde_json::Value = serde_json::from_str(&text)?;

        // Verify response has expected fields
        assert!(
            transfer_response.get("tx_hash").is_some(),
            "Response should have tx_hash"
        );

        // Verify the transfer actually happened by checking the balance
        let final_balance = fixture
            .provider
            .get_asset_balance(&random_wallet.address(), base_asset_id)
            .await?;

        assert_eq!(
            final_balance,
            initial_balance + transfer_amount as u128,
            "Recipient balance should increase by transfer amount"
        );

        Ok(())
    }

    #[tokio::test]
    async fn test_transfer_assets_to_contract_http_mcp() -> Result<()> {
        let fixture = E2ETestFixture::new().await.unwrap();
        let mut client = ForcMcpClient::http_stream_client().await?;

        // Transfer to the contract itself
        let consensus_parameters = fixture.provider.consensus_parameters().await?;
        let base_asset_id = consensus_parameters.base_asset_id();

        // Parse contract ID to get the ContractId type
        let contract_id = ContractId::from_str(&fixture.contract_id)
            .map_err(|e| anyhow::anyhow!("Failed to parse contract ID: {}", e))?;

        let initial_balance = fixture
            .provider
            .get_contract_asset_balance(&contract_id, base_asset_id)
            .await?;

        // Test transferring assets to contract through MCP SSE server
        let transfer_amount = 500u64;
        let args = fixture.create_transfer_tool_args(&fixture.contract_id, transfer_amount);

        let result = client.call_tool("transfer_assets", args).await?;

        assert!(!result.content.is_empty(), "Content should not be empty");

        // Verify the transfer actually happened
        let final_balance = fixture
            .provider
            .get_contract_asset_balance(&contract_id, base_asset_id)
            .await?;

        assert_eq!(
            final_balance,
            initial_balance + transfer_amount,
            "Contract balance should increase by transfer amount"
        );

        Ok(())
    }

    #[tokio::test]
    async fn test_error_handling_invalid_contract_id_http_mcp() -> Result<()> {
        let fixture = E2ETestFixture::new().await.unwrap();
        let mut client = ForcMcpClient::http_stream_client().await?;

        // Test with invalid contract ID
        let mut args = fixture.create_call_tool_args("test_u8", vec!["255"]);
        args.insert(
            "contract_id".to_string(),
            serde_json::Value::String("invalid_contract_id".to_string()),
        );

        let result = client.call_tool("call_contract", args).await?;

        assert_eq!(
            result.is_error,
            Some(true),
            "Should return an error result for invalid contract ID"
        );

        Ok(())
    }

    #[tokio::test]
    async fn test_error_handling_missing_function_http_mcp() -> Result<()> {
        let fixture = E2ETestFixture::new().await.unwrap();
        let mut client = ForcMcpClient::http_stream_client().await?;

        // Test with non-existent function
        let args = fixture.create_call_tool_args("non_existent_function", vec![]);

        let result = client.call_tool("call_contract", args).await?;

        assert_eq!(
            result.is_error,
            Some(true),
            "Should return an error result for missing function"
        );

        Ok(())
    }

    #[tokio::test]
    async fn test_error_handling_invalid_transfer_amount_http_mcp() -> Result<()> {
        let fixture = E2ETestFixture::new().await.unwrap();
        let mut client = ForcMcpClient::http_stream_client().await?;

        // Create recipient
        let random_wallet = Wallet::random(&mut rand::thread_rng(), fixture.provider.clone());
        let recipient_address = format!("0x{}", random_wallet.address());

        // Test with amount that exceeds wallet balance
        let excessive_amount = 1_000_000_000_000u64; // Way more than wallet has
        let args = fixture.create_transfer_tool_args(&recipient_address, excessive_amount);

        let result = client.call_tool("transfer_assets", args).await?;

        assert_eq!(
            result.is_error,
            Some(true),
            "Should return an error result for insufficient funds"
        );

        Ok(())
    }

    #[tokio::test]
    async fn test_get_execution_trace_tool_http_mcp() -> Result<()> {
        let fixture = E2ETestFixture::new().await.unwrap();
        let mut client = ForcMcpClient::http_stream_client().await?;

        // First, get a call response with trace events by calling a contract function
        let call_args = fixture.create_call_tool_args("test_u8", vec!["42"]);
        let call_result = client.call_tool("call_contract", call_args).await?;

        assert!(
            !call_result.content.is_empty(),
            "Call result should not be empty"
        );

        // Extract the CallResponse from the call result
        let call_text = E2ETestFixture::extract_text_from_content(&call_result.content[0])
            .expect("Call response should be text");
        let call_response: serde_json::Value = serde_json::from_str(&call_text)?;

        // Extract trace events from the response
        let trace_events = call_response
            .get("trace_events")
            .expect("Call response should have trace_events")
            .clone();

        let total_gas = call_response
            .get("total_gas")
            .expect("Call response should have total_gas")
            .as_u64()
            .expect("total_gas should be a number");

        // Convert trace_events to array of JSON objects for the trace tool
        let trace_events_array: Vec<Value> = if let Some(events_array) = trace_events.as_array() {
            events_array.clone()
        } else {
            vec![]
        };

        // Create arguments for the get_execution_trace tool
        let mut trace_args = HashMap::new();
        trace_args.insert("trace_events".to_string(), Value::Array(trace_events_array));
        trace_args.insert("total_gas".to_string(), Value::Number(total_gas.into()));

        // Call the get_execution_trace tool
        let trace_result = client.call_tool("get_execution_trace", trace_args).await?;

        assert_eq!(
            trace_result.is_error,
            Some(false),
            "Trace tool should not error"
        );
        assert!(
            !trace_result.content.is_empty(),
            "Trace result should not be empty"
        );

        // Extract and validate the trace output
        let trace_output = E2ETestFixture::extract_text_from_content(&trace_result.content[0])
            .expect("Trace result should be text");

        // Verify the trace output contains expected elements
        assert!(
            trace_output.contains("Traces:"),
            "Output should contain 'Traces:'"
        );
        assert!(
            trace_output.contains("[Script]"),
            "Output should contain '[Script]'"
        );
        assert!(
            trace_output.contains("Gas used:"),
            "Output should contain 'Gas used:'"
        );

        // Should contain contract calls or returns
        assert!(
            trace_output.contains("├─") || trace_output.contains("└─"),
            "Output should contain trace tree structure"
        );

        Ok(())
    }

    #[tokio::test]
    async fn test_get_execution_trace_with_labels_http_mcp() -> Result<()> {
        let fixture = E2ETestFixture::new().await.unwrap();
        let mut client = ForcMcpClient::http_stream_client().await?;

        // First, get a call response with trace events
        let call_args = fixture.create_call_tool_args("test_empty", vec![]);
        let call_result = client.call_tool("call_contract", call_args).await?;

        let call_text = E2ETestFixture::extract_text_from_content(&call_result.content[0])
            .expect("Call response should be text");
        let call_response: serde_json::Value = serde_json::from_str(&call_text)?;

        let trace_events = call_response["trace_events"].clone();
        let total_gas = call_response["total_gas"].as_u64().unwrap();

        // Create a labels map for testing
        let mut labels = std::collections::HashMap::new();
        let contract_id = fuels_core::types::ContractId::from_str(&fixture.contract_id)
            .map_err(|e| anyhow::anyhow!("Failed to parse contract ID: {}", e))?;
        labels.insert(contract_id, "TestContract".to_string());

        // Convert HashMap<ContractId, String> to HashMap<String, String>
        let labels_map: HashMap<String, String> = labels
            .iter()
            .map(|(contract_id, label)| (format!("0x{}", contract_id), label.clone()))
            .collect();

        // Convert trace_events to array of JSON objects
        let trace_events_array: Vec<Value> = if let Some(events_array) = trace_events.as_array() {
            events_array.clone()
        } else {
            vec![]
        };

        // Create arguments with labels
        let mut trace_args = HashMap::new();
        trace_args.insert("trace_events".to_string(), Value::Array(trace_events_array));
        trace_args.insert("total_gas".to_string(), Value::Number(total_gas.into()));
        trace_args.insert("labels".to_string(), serde_json::to_value(labels_map)?);

        // Call the get_execution_trace tool with labels
        let trace_result = client.call_tool("get_execution_trace", trace_args).await?;

        assert_eq!(
            trace_result.is_error,
            Some(false),
            "Trace tool should not error"
        );
        assert!(
            !trace_result.content.is_empty(),
            "Trace result should not be empty"
        );

        let trace_output = E2ETestFixture::extract_text_from_content(&trace_result.content[0])
            .expect("Trace result should be text");

        // Verify the trace output uses the label
        assert!(
            trace_output.contains("TestContract"),
            "Output should contain the contract label"
        );

        Ok(())
    }

    #[tokio::test]
    async fn test_get_execution_trace_error_handling_http_mcp() -> Result<()> {
        let mut client = ForcMcpClient::http_stream_client().await?;

        // Test with invalid trace_events JSON (malformed object)
        let mut invalid_args = HashMap::new();
        let mut invalid_trace_event = HashMap::new();
        invalid_trace_event.insert(
            "invalid_field".to_string(),
            Value::String("invalid_value".to_string()),
        );
        invalid_args.insert(
            "trace_events".to_string(),
            Value::Array(vec![serde_json::to_value(invalid_trace_event).unwrap()]),
        );
        invalid_args.insert("total_gas".to_string(), Value::Number(1000.into()));

        let result = client
            .call_tool("get_execution_trace", invalid_args)
            .await?;

        assert_eq!(
            result.is_error,
            Some(true),
            "Should return error for invalid trace_events JSON"
        );

        // Test with invalid labels JSON (invalid contract ID)
        let mut invalid_labels_args = HashMap::new();
        invalid_labels_args.insert("trace_events".to_string(), Value::Array(vec![]));
        invalid_labels_args.insert("total_gas".to_string(), Value::Number(1000.into()));
        let mut invalid_labels_map = HashMap::new();
        invalid_labels_map.insert("invalid_contract_id".to_string(), "TestLabel".to_string());
        invalid_labels_args.insert(
            "labels".to_string(),
            serde_json::to_value(invalid_labels_map).unwrap(),
        );

        let result = client
            .call_tool("get_execution_trace", invalid_labels_args)
            .await?;

        assert_eq!(
            result.is_error,
            Some(true),
            "Should return error for invalid labels JSON"
        );

        Ok(())
    }
}
