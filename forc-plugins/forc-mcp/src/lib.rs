pub mod auth;
pub mod forc_call;
pub mod rate_limit;

use axum::{
    extract::State,
    response::IntoResponse,
    routing::{get, post},
    Router,
};
use rate_limit::{public_rate_limit_middleware, RateLimitConfig, RateLimiter};
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
        Box::pin(async move { Err(McpError::resource_not_found("Resource not found", None)) })
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
        Err(McpError::resource_not_found("Resource not found", None))
    }
}

// Server runner functions
pub async fn run_stdio_server(server: ForcMcpServer) -> anyhow::Result<()> {
    tracing::info!("Starting MCP server in STDIO mode");

    let server_handler = server.serve(stdio()).await?;

    tracing::info!("MCP server started successfully in STDIO mode");
    server_handler.waiting().await?;
    Ok(())
}

pub async fn run_sse_server(server: ForcMcpServer, port: Option<u16>) -> anyhow::Result<()> {
    let port = match port {
        Some(p) => p,
        None => find_available_port().await?,
    };

    tracing::info!("Starting MCP SSE server on port {port}");
    let bind_addr = format!("0.0.0.0:{port}").parse()?;
    let ct = SseServer::serve(bind_addr)
        .await?
        .with_service(move || server.clone());

    tracing::info!("MCP SSE server started successfully on port: {port}");
    tracing::info!("SSE endpoint: /sse");
    tracing::info!("Messages endpoint: /message");

    tokio::signal::ctrl_c().await?;
    ct.cancel();

    tracing::info!("MCP SSE server shut down successfully");
    Ok(())
}

pub async fn run_http_server(
    server: ForcMcpServer,
    port: Option<u16>,
    auth_config: auth::AuthConfig,
) -> anyhow::Result<()> {
    let port = match port {
        Some(p) => p,
        None => find_available_port().await?,
    };

    tracing::info!("Starting MCP HTTP streamable server on port {port}");
    let bind_addr = format!("0.0.0.0:{port}");

    let auth_manager = if auth_config.enabled {
        Some(Arc::new(auth::AuthManager::new(auth_config.clone()).await?))
    } else {
        None
    };

    let service = StreamableHttpService::new(
        move || Ok(server.clone()),
        LocalSessionManager::default().into(),
        Default::default(),
    );

    // Create separate rate limiters for public and authenticated requests
    let public_rate_limiter = Arc::new(RateLimiter::new(RateLimitConfig {
        requests_per_minute: auth_config.public_rate_limit_per_minute,
        requests_per_day: auth_config.public_rate_limit_per_day,
    }));
    let api_key_rate_limiter = Arc::new(RateLimiter::new(RateLimitConfig {
        requests_per_minute: auth_config.api_key_rate_limit_per_minute,
        requests_per_day: auth_config.api_key_rate_limit_per_day,
    }));

    // Spawn cleanup task for rate limiters
    let public_limiter_cleanup = public_rate_limiter.clone();
    let api_key_limiter_cleanup = api_key_rate_limiter.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(300)); // 5 minutes
        loop {
            interval.tick().await;
            public_limiter_cleanup.cleanup_expired_trackers().await;
            api_key_limiter_cleanup.cleanup_expired_trackers().await;
        }
    });

    let mut router = Router::new().route("/health", get(|| async { "OK" }));

    tracing::info!("MCP endpoint: /mcp");
    if let Some(auth_mgr) = &auth_manager {
        tracing::info!("Authentication enabled");

        // Single /mcp endpoint with unified auth and rate limiting
        router = router
            .nest_service("/mcp", service.clone())
            .layer(axum::middleware::from_fn_with_state(
                auth_mgr.clone(),
                unified_api_key_auth_middleware,
            ))
            .layer(axum::Extension(public_rate_limiter.clone()))
            .layer(axum::Extension(api_key_rate_limiter.clone()));

        if !auth_config.api_keys_only {
            tracing::info!(
                "Public rate limits: {}/min, {}/day",
                auth_config.public_rate_limit_per_minute,
                auth_config.public_rate_limit_per_day
            );
        }
        tracing::info!(
            "API key rate limits: {}/min, {}/day",
            auth_config.api_key_rate_limit_per_minute,
            auth_config.api_key_rate_limit_per_day
        );

        // Admin routes with authentication
        let admin_routes = Router::new()
            .route(
                "/api-keys",
                post(auth::create_api_key).get(auth::list_api_keys),
            )
            .route(
                "/api-keys/{key_id}",
                get(auth::get_api_key).delete(auth::delete_api_key),
            )
            .route("/import", post(auth::import_api_keys))
            .layer(axum::middleware::from_fn_with_state(
                auth_mgr.clone(),
                admin_auth_middleware,
            ))
            .with_state(auth_mgr.clone());
        router = router.nest("/admin", admin_routes);
        tracing::info!("Admin endpoint: /admin/* (requires X-API-Key: <admin-api-key> header)");
    } else {
        // No auth, just basic service with public rate limiting
        router = router
            .nest_service("/mcp", service)
            .layer(axum::middleware::from_fn(public_rate_limit_middleware))
            .layer(axum::Extension(public_rate_limiter.clone()));
        tracing::info!("Authentication disabled - public endpoint only");
        tracing::info!(
            "Public rate limits: {}/min, {}/day",
            auth_config.public_rate_limit_per_minute,
            auth_config.public_rate_limit_per_day
        );
    }

    let tcp_listener = tokio::net::TcpListener::bind(bind_addr).await?;

    tracing::info!("MCP HTTP streamable server started successfully on port: {port}");

    // Run the server with proper connection info for IP extraction
    axum::serve(
        tcp_listener,
        router.into_make_service_with_connect_info::<std::net::SocketAddr>(),
    )
    .with_graceful_shutdown(async {
        tokio::signal::ctrl_c()
            .await
            .expect("Failed to install CTRL+C signal handler");
        tracing::info!("MCP HTTP streamable server shutting down...");
    })
    .await
    .map_err(|e| anyhow::anyhow!("Failed to serve HTTP streamable server: {}", e))?;

    Ok(())
}

/// Unified authentication middleware for /mcp endpoint
/// Handles both public and authenticated requests based on auth_only setting
async fn unified_api_key_auth_middleware(
    State(auth_manager): axum::extract::State<Arc<auth::AuthManager>>,
    req: axum::extract::Request,
    next: axum::middleware::Next,
) -> Result<axum::response::Response, axum::response::Response> {
    let headers = req.headers();
    let api_key = auth::extract_api_key(headers);

    // Check if api_keys_only mode is enabled (get from config through auth_manager)
    let api_keys_only = auth_manager.config.api_keys_only;
    match (api_key, api_keys_only) {
        // API key provided - validate and track usage
        (Some(key), _) => {
            match auth_manager.check_and_track_usage(&key).await {
                Ok(Some(_)) => {
                    // Valid API key with rate limit check passed
                    Ok(next.run(req).await)
                }
                Ok(None) => Err((
                    axum::http::StatusCode::UNAUTHORIZED,
                    axum::Json(auth::ErrorResponse {
                        error: "Invalid API key".to_string(),
                    }),
                )
                    .into_response()),
                Err(e) => {
                    // Check if it's a rate limit error
                    let error_msg = e.to_string();
                    if error_msg.contains("Rate limit exceeded") {
                        Err((
                            axum::http::StatusCode::TOO_MANY_REQUESTS,
                            axum::Json(auth::ErrorResponse { error: error_msg }),
                        )
                            .into_response())
                    } else {
                        Err((
                            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                            axum::Json(auth::ErrorResponse {
                                error: "Internal server error".to_string(),
                            }),
                        )
                            .into_response())
                    }
                }
            }
        }
        // No API key, but api_keys_only mode - reject
        (None, true) => Err((
            axum::http::StatusCode::UNAUTHORIZED,
            axum::Json(auth::ErrorResponse {
                error: "X-API-Key header required".to_string(),
            }),
        )
            .into_response()),
        // No API key, public access allowed - proceed with public rate limits
        (None, false) => Ok(next.run(req).await),
    }
}

/// Admin authentication middleware
async fn admin_auth_middleware(
    State(auth_manager): axum::extract::State<Arc<auth::AuthManager>>,
    req: axum::extract::Request,
    next: axum::middleware::Next,
) -> Result<axum::response::Response, axum::response::Response> {
    // Extract API key from X-API-Key header
    let headers = req.headers();
    let api_key = auth::extract_api_key(headers);

    if let Some(key) = api_key {
        match auth_manager.check_and_track_usage(&key).await {
            Ok(Some(api_key)) if api_key.role == auth::Role::Admin => Ok(next.run(req).await),
            Ok(Some(_)) => Err((
                axum::http::StatusCode::FORBIDDEN,
                axum::Json(auth::ErrorResponse {
                    error: "Admin access required".to_string(),
                }),
            )
                .into_response()),
            Ok(None) => Err((
                axum::http::StatusCode::UNAUTHORIZED,
                axum::Json(auth::ErrorResponse {
                    error: "Invalid API key".to_string(),
                }),
            )
                .into_response()),
            Err(e) => Err((
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                axum::Json(auth::ErrorResponse {
                    error: format!("Internal server error: {}", e),
                }),
            )
                .into_response()),
        }
    } else {
        Err((
            axum::http::StatusCode::UNAUTHORIZED,
            axum::Json(auth::ErrorResponse {
                error: "X-API-Key header required".to_string(),
            }),
        )
            .into_response())
    }
}

async fn find_available_port() -> anyhow::Result<u16> {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await?;
    let addr = listener.local_addr()?;
    Ok(addr.port())
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use anyhow::{anyhow, Result};
    use forc_call::ForcCallTools;
    use rmcp::model::{CallToolRequestParam, ClientCapabilities, ClientInfo, Implementation};
    use rmcp::transport::{sse_client::SseClientTransport, StreamableHttpClientTransport};
    use rmcp::{ServerHandler, ServiceExt};
    use tokio::time::{sleep, Duration};

    /// Unified test utility for running e2e tests against MCP servers
    pub struct ForcMcpClient {
        mcp_client: rmcp::service::RunningService<rmcp::service::RoleClient, ClientInfo>,
        server_handle: tokio::task::JoinHandle<Result<()>>,
    }

    impl ForcMcpClient {
        /// Create a new MCP SSE test client
        pub async fn sse_client() -> Result<Self> {
            let port = find_available_port().await?;

            // Start the SSE server in a background task with the specific port
            let server = ForcMcpServer::new().register_module(ForcCallTools::new());
            let server_handle =
                tokio::spawn(async move { run_sse_server(server, Some(port)).await });

            // Wait a bit for the server to start
            sleep(Duration::from_millis(100)).await;

            // Check if server is still running
            if server_handle.is_finished() {
                return Err(anyhow!("Server task completed before test could run"));
            }

            let base_url = format!("http://127.0.0.1:{}", port);

            // Create MCP client using SSE transport
            let transport = SseClientTransport::start(format!("{}/sse", base_url)).await?;
            let client_info = ClientInfo {
                protocol_version: Default::default(),
                capabilities: ClientCapabilities::default(),
                client_info: Implementation {
                    name: "forc-mcp-sse-client".to_string(),
                    version: "0.1.0".to_string(),
                },
            };
            let mcp_client = client_info.serve(transport).await?;

            let test_client = ForcMcpClient {
                mcp_client,
                server_handle,
            };

            Ok(test_client)
        }

        /// Create a new MCP HTTP streamable test client
        pub async fn http_stream_client() -> Result<Self> {
            let port = find_available_port().await?;

            // Start the HTTP server in a background task with the specific port
            let server = ForcMcpServer::new().register_module(ForcCallTools::new());
            let server_handle = tokio::spawn(async move {
                run_http_server(server, Some(port), auth::AuthConfig::default()).await
            });

            // Wait a bit for the server to start
            sleep(Duration::from_millis(100)).await;

            // Check if server is still running
            if server_handle.is_finished() {
                return Err(anyhow!("Server task completed before test could run"));
            }

            let base_url = format!("http://127.0.0.1:{}/mcp", port);

            // Create MCP client using HTTP streamable transport
            let transport = StreamableHttpClientTransport::from_uri(base_url);
            let client_info = ClientInfo {
                protocol_version: Default::default(),
                capabilities: ClientCapabilities::default(),
                client_info: Implementation {
                    name: "forc-mcp-http-client".to_string(),
                    version: "0.1.0".to_string(),
                },
            };
            let mcp_client = client_info.serve(transport).await?;

            let test_client = ForcMcpClient {
                mcp_client,
                server_handle,
            };

            Ok(test_client)
        }

        pub async fn list_tools(&mut self) -> Result<Vec<String>> {
            let tools = self.mcp_client.list_tools(Default::default()).await?;
            Ok(tools
                .tools
                .into_iter()
                .map(|tool| tool.name.to_string())
                .collect())
        }

        pub async fn call_tool(
            &mut self,
            tool_name: &str,
            arguments: std::collections::HashMap<String, serde_json::Value>,
        ) -> Result<rmcp::model::CallToolResult> {
            let param = CallToolRequestParam {
                name: tool_name.to_string().into(),
                arguments: Some(arguments.into_iter().collect()),
            };
            let result = self.mcp_client.call_tool(param).await?;
            Ok(result)
        }

        pub async fn list_resources(&mut self) -> Result<Vec<String>> {
            let resources = self.mcp_client.list_resources(Default::default()).await?;
            Ok(resources
                .resources
                .into_iter()
                .map(|resource| resource.raw.uri)
                .collect())
        }

        pub async fn read_resource(&mut self, uri: &str) -> Result<String> {
            let param = ReadResourceRequestParam {
                uri: uri.to_string(),
            };
            let result = self.mcp_client.read_resource(param).await?;
            if let Some(content) = result.contents.first() {
                // Extract text from ResourceContents
                let json_value = serde_json::to_value(content)?;
                if let Some(text) = json_value.get("text") {
                    if let Some(text_str) = text.as_str() {
                        return Ok(text_str.to_string());
                    }
                }
            }
            Err(anyhow!("No text content found in resource"))
        }
    }

    impl Drop for ForcMcpClient {
        fn drop(&mut self) {
            self.server_handle.abort();
        }
    }

    #[tokio::test]
    async fn test_server_info() -> Result<()> {
        let server = ForcMcpServer::new().register_module(ForcCallTools::new());
        let info = server.get_info();

        assert_eq!(info.server_info.name, "forc-mcp-server");
        assert!(info.capabilities.tools.is_some());
        assert!(info.capabilities.resources.is_some());
        assert!(info.instructions.is_some());
        assert!(info.instructions.unwrap().contains("forc-call-tools"));

        Ok(())
    }

    #[test]
    fn test_server_creation() {
        let server = ForcMcpServer::new().register_module(ForcCallTools::new());
        assert_eq!(server.get_info().server_info.name, "forc-mcp-server");
        assert_eq!(
            server.get_info().instructions.unwrap(),
            "Forc MCP server with modules: forc-call-tools"
        );
    }

    #[tokio::test]
    async fn test_unified_client_both_transports() -> Result<()> {
        // Test SSE client
        let mut sse_client = ForcMcpClient::sse_client().await?;
        let sse_tools = sse_client.list_tools().await?;

        // Test HTTP streamable client
        let mut http_client = ForcMcpClient::http_stream_client().await?;
        let http_tools = http_client.list_tools().await?;

        // Both clients should expose the same tools
        assert_eq!(sse_tools.len(), http_tools.len());
        assert!(sse_tools.contains(&"list_contract_functions".to_string()));
        assert!(http_tools.contains(&"list_contract_functions".to_string()));
        assert!(sse_tools.contains(&"call_contract".to_string()));
        assert!(http_tools.contains(&"call_contract".to_string()));
        assert!(sse_tools.contains(&"transfer_assets".to_string()));
        assert!(http_tools.contains(&"transfer_assets".to_string()));
        assert!(sse_tools.contains(&"get_execution_trace".to_string()));
        assert!(http_tools.contains(&"get_execution_trace".to_string()));

        Ok(())
    }
}
