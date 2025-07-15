
use rmcp::{
    model::*,
    service::RequestContext,
    transport::{
        sse_server::SseServer,
        stdio,
        streamable_http_server::{session::local::LocalSessionManager, StreamableHttpService},
    },
    Error as McpError, RoleServer, ServiceExt,
};
use std::{future::Future, pin::Pin, sync::Arc};
use tracing::info;

/// Trait that all MCP tool modules must implement to be registered with ForcMcpServer
///
/// This trait provides a common interface for all tool modules, allowing them to be
/// registered and managed by the main MCP server.
pub trait McpToolModule: Send + Sync + 'static {
    /// Get the name of this tool module
    fn get_module_name(&self) -> &'static str;

    /// List all tools provided by this module
    fn list_tools(
        &self,
        request: Option<PaginatedRequestParam>,
        ctx: RequestContext<RoleServer>,
    ) -> Pin<Box<dyn Future<Output = Result<ListToolsResult, McpError>> + Send>>;

    /// Handle a tool call for this module
    fn call_tool(
        &self,
        request: CallToolRequestParam,
        ctx: RequestContext<RoleServer>,
    ) -> Pin<Box<dyn Future<Output = Result<CallToolResult, McpError>> + Send>>;

    /// List all resources provided by this module (optional)
    fn list_resources(
        &self,
        _request: Option<PaginatedRequestParam>,
        _ctx: RequestContext<RoleServer>,
    ) -> Pin<Box<dyn Future<Output = Result<ListResourcesResult, McpError>> + Send>> {
        Box::pin(async move {
            Ok(ListResourcesResult {
                resources: vec![],
                next_cursor: None,
            })
        })
    }

    /// Read a resource from this module (optional)
    fn read_resource(
        &self,
        _request: ReadResourceRequestParam,
        _ctx: RequestContext<RoleServer>,
    ) -> Pin<Box<dyn Future<Output = Result<ReadResourceResult, McpError>> + Send>> {
        Box::pin(async move {
            Err(McpError::resource_not_found(
                "Resource not found",
                None,
            ))
        })
    }

    /// Get server info for this module
    fn get_info(&self) -> ServerInfo;
}

#[derive(Clone, Default)]
pub struct ForcMcpServer {
    tool_handlers: Vec<Arc<dyn McpToolModule>>,
}

impl ForcMcpServer {
    /// Create a new empty MCP server
    ///
    /// Tool modules must be registered explicitly using `register_module()`
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a tool module with the server
    ///
    /// This allows the server to route tool calls to the appropriate module.
    pub fn register_module<T: McpToolModule + 'static>(mut self, module: T) -> Self {
        self.tool_handlers.push(Arc::new(module));
        self
    }
}

impl rmcp::ServerHandler for ForcMcpServer {
    fn get_info(&self) -> ServerInfo {
        let module_names = self
            .tool_handlers
            .iter()
            .map(|handler| handler.get_module_name().to_string())
            .collect::<Vec<String>>();

        ServerInfo {
            protocol_version: ProtocolVersion::V_2024_11_05,
            capabilities: ServerCapabilities::builder()
                .enable_tools()
                .enable_resources()
                .build(),
            server_info: Implementation {
                name: "forc-mcp-server".to_string(),
                version: env!("CARGO_PKG_VERSION").to_string(),
            },
            instructions: Some(format!(
                "Forc MCP server with modules: {}",
                module_names.join(", ")
            )),
        }
    }

    async fn list_tools(
        &self,
        request: Option<PaginatedRequestParam>,
        ctx: RequestContext<RoleServer>,
    ) -> Result<ListToolsResult, McpError> {
        let mut tools = Vec::new();
        for handler in &self.tool_handlers {
            let result = handler.list_tools(request.clone(), ctx.clone()).await?;
            tools.extend(result.tools);
        }
        Ok(ListToolsResult {
            tools,
            next_cursor: None,
        })
    }

    async fn call_tool(
        &self,
        request: CallToolRequestParam,
        ctx: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        let tool_name = request.name.to_string();
        // Find the module that has this tool
        for handler in &self.tool_handlers {
            let tools_result = handler.list_tools(None, ctx.clone()).await?;
            if tools_result.tools.iter().any(|tool| tool.name == tool_name) {
                return handler.call_tool(request, ctx).await;
            }
        }
        Err(McpError::method_not_found::<CallToolRequestMethod>())
    }

    async fn list_resources(
        &self,
        request: Option<PaginatedRequestParam>,
        ctx: RequestContext<RoleServer>,
    ) -> Result<ListResourcesResult, McpError> {
        let mut resources = Vec::new();
        for handler in &self.tool_handlers {
            let result = handler.list_resources(request.clone(), ctx.clone()).await?;
            resources.extend(result.resources);
        }
        Ok(ListResourcesResult {
            resources,
            next_cursor: None,
        })
    }

    async fn read_resource(
        &self,
        request: ReadResourceRequestParam,
        ctx: RequestContext<RoleServer>,
    ) -> Result<ReadResourceResult, McpError> {
        // Try each handler until one successfully reads the resource
        for handler in &self.tool_handlers {
            match handler.read_resource(request.clone(), ctx.clone()).await {
                Ok(result) => return Ok(result),
                Err(e) => {
                    // Check if this is a resource_not_found error
                    if let Ok(json) = serde_json::to_value(&e) {
                        if let Some(error) = json.get("error") {
                            if let Some(code) = error.get("code") {
                                if code == "resource_not_found" {
                                    // Continue to next handler
                                    continue;
                                }
                            }
                        }
                    }
                    return Err(e);
                }
            }
        }
        Err(McpError::resource_not_found(
            "Resource not found",
            None,
        ))
    }
}

// Server runner functions
pub async fn run_stdio_server(server: ForcMcpServer) -> anyhow::Result<()> {
    info!("Starting MCP server in STDIO mode");

    let server_handler = server.serve(stdio()).await?;

    info!("MCP server started successfully in STDIO mode");
    server_handler.waiting().await?;
    Ok(())
}

pub async fn run_sse_server(server: ForcMcpServer, port: Option<u16>) -> anyhow::Result<()> {
    let port = match port {
        Some(p) => p,
        None => find_available_port().await?,
    };

    info!("Starting MCP SSE server on port {port}");
    let bind_addr = format!("0.0.0.0:{port}").parse()?;
    let ct = SseServer::serve(bind_addr)
        .await?
        .with_service(move || server.clone());

    info!("MCP SSE server started successfully on port: {port}");
    info!("SSE endpoint: /sse");
    info!("Messages endpoint: /message");

    tokio::signal::ctrl_c().await?;
    ct.cancel();

    info!("MCP SSE server shut down successfully");
    Ok(())
}

pub async fn run_http_server(server: ForcMcpServer, port: Option<u16>) -> anyhow::Result<()> {
    let port = match port {
        Some(p) => p,
        None => find_available_port().await?,
    };

    info!("Starting MCP HTTP streamable server on port {port}");
    let bind_addr = format!("0.0.0.0:{port}");

    let service = StreamableHttpService::new(
        move || Ok(server.clone()),
        LocalSessionManager::default().into(),
        Default::default(),
    );

    let router = axum::Router::new().nest_service("/mcp", service);
    let tcp_listener = tokio::net::TcpListener::bind(bind_addr).await?;

    info!("MCP HTTP streamable server started successfully on port: {port}");
    info!("HTTP endpoint: /mcp");

    // Run the server
    axum::serve(tcp_listener, router)
        .with_graceful_shutdown(async {
            tokio::signal::ctrl_c()
                .await
                .expect("Failed to install CTRL+C signal handler");
            info!("MCP HTTP streamable server shutting down...");
        })
        .await
        .map_err(|e| anyhow::anyhow!("Failed to serve HTTP streamable server: {}", e))?;

    Ok(())
}

async fn find_available_port() -> anyhow::Result<u16> {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await?;
    let addr = listener.local_addr()?;
    Ok(addr.port())
}
