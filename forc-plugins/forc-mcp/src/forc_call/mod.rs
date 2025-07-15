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
    pub trace_events: Vec<HashMap<String, Value>>, // JSON objects representing TraceEvent
    pub total_gas: u64,
    pub labels: Option<HashMap<String, String>>, // JSON string representation of HashMap<ContractId, String>
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
