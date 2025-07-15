use rmcp::{model::*, service::RequestContext, Error as McpError, RoleServer};

// Resource URI constants
pub const TYPE_ENCODING_REFERENCE_URI: &str = "forc-call://type-encoding-reference";
pub const COMMON_COMMANDS_URI: &str = "forc-call://examples/common-commands";
pub const CONTRACT_SAMPLES_URI: &str = "forc-call://examples/contract-samples";

/// Get the type encoding reference content
pub fn get_type_encoding_reference() -> &'static str {
    include_str!("../../../../docs/book/src/forc/plugins/forc_mcp/tools/forc_call/type_encoding_reference.md")
}

/// Get the common commands content
pub fn get_common_commands() -> &'static str {
    include_str!("../../../../docs/book/src/forc/plugins/forc_mcp/tools/forc_call/common_commands.md")
}

/// Get the contract samples content
pub fn get_contract_samples() -> &'static str {
    include_str!("../../../../docs/book/src/forc/plugins/forc_mcp/tools/forc_call/contract_samples.md")
}

/// Handle resource read requests
pub async fn read_resource(
    uri: &str,
    _: RequestContext<RoleServer>,
) -> Result<ReadResourceResult, McpError> {
    match uri {
        TYPE_ENCODING_REFERENCE_URI => {
            let content = get_type_encoding_reference();
            Ok(ReadResourceResult {
                contents: vec![ResourceContents::text(content, uri)],
            })
        }
        COMMON_COMMANDS_URI => {
            let content = get_common_commands();
            Ok(ReadResourceResult {
                contents: vec![ResourceContents::text(content, uri)],
            })
        }
        CONTRACT_SAMPLES_URI => {
            let content = get_contract_samples();
            Ok(ReadResourceResult {
                contents: vec![ResourceContents::text(content, uri)],
            })
        }
        _ => Err(McpError::resource_not_found(
            "Resource not found",
            Some(serde_json::json!({
                "uri": uri
            })),
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::{COMMON_COMMANDS_URI, CONTRACT_SAMPLES_URI, TYPE_ENCODING_REFERENCE_URI};
    use crate::tests::ForcMcpClient;
    use anyhow::Result;

    #[tokio::test]
    async fn test_forc_call_resources() -> Result<()> {
        let mut client = ForcMcpClient::http_stream_client().await?;

        // List resources
        let resources = client.list_resources().await?;
        assert_eq!(resources.len(), 3);
        assert!(resources.contains(&TYPE_ENCODING_REFERENCE_URI.to_string()));
        assert!(resources.contains(&COMMON_COMMANDS_URI.to_string()));
        assert!(resources.contains(&CONTRACT_SAMPLES_URI.to_string()));

        // Read type encoding reference
        let type_ref = client.read_resource(TYPE_ENCODING_REFERENCE_URI).await?;
        assert!(type_ref.contains("MCP Tool Type Encoding Reference"));
        assert!(type_ref.contains("bool"));
        assert!(type_ref.contains("`u8`, `u16`, `u32`, `u64`"));
        assert!(type_ref.contains("Structs are encoded as tuples"));
        assert!(type_ref.contains("call_contract"));

        // Read common commands
        let commands = client.read_resource(COMMON_COMMANDS_URI).await?;
        assert!(commands.contains("Common MCP Tool Usage"));
        assert!(commands.contains("\"mode\": \"dry-run\""));
        assert!(commands.contains("\"mode\": \"simulate\""));
        assert!(commands.contains("\"mode\": \"live\""));
        assert!(commands.contains("\"tool\": \"call_contract\""));

        // Read contract samples
        let samples = client.read_resource(CONTRACT_SAMPLES_URI).await?;
        assert!(samples.contains("Contract Examples with MCP Tool Usage"));
        assert!(samples.contains("Simple Counter Contract"));
        assert!(samples.contains("Token Contract"));
        assert!(samples.contains("Complex Types Contract"));
        assert!(samples.contains("MCP Tool Commands"));

        Ok(())
    }

    #[tokio::test]
    async fn test_resource_not_found() -> Result<()> {
        let mut client = ForcMcpClient::http_stream_client().await?;

        // Try to read non-existent resource
        let result = client.read_resource("forc-call://non-existent").await;
        assert!(result.is_err());

        Ok(())
    }
}
