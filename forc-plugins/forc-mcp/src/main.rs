use anyhow::Result;
use clap::{Parser, Subcommand};
use rmcp::{transport::stdio, ServiceExt};
use tracing::info;

/// Model Context Protocol (MCP) server for Forc
#[derive(Parser)]
#[command(name = "forc-mcp")]
#[command(about = "MCP server plugin for Forc")]
#[command(version = env!("CARGO_PKG_VERSION"))]
#[command(subcommand_required = true)]
#[command(arg_required_else_help = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Run MCP server in STDIO mode
    Stdio,
    /// Run MCP server in SSE mode
    Sse {
        /// Port to bind the SSE server to
        #[arg(short, long, default_value = "3000")]
        port: u16,
    },
}

#[derive(Clone, Default)]
pub struct ForcMcpServer;

impl ForcMcpServer {
    const fn new() -> Self {
        Self
    }
}

impl rmcp::ServerHandler for ForcMcpServer {
    fn get_info(&self) -> rmcp::model::InitializeResult {
        rmcp::model::InitializeResult {
            protocol_version: rmcp::model::ProtocolVersion::V_2024_11_05,
            capabilities: rmcp::model::ServerCapabilities {
                logging: None,
                prompts: None,
                resources: None,
                tools: None,
                completions: None,
                experimental: None,
            },
            server_info: rmcp::model::Implementation {
                name: "forc-mcp".into(),
                version: "0.1.0".into(),
            },
            instructions: Some("A Model Context Protocol server for Forc toolchain".into()),
        }
    }
}

async fn run_stdio_server() -> Result<()> {
    info!("Starting MCP server in STDIO mode");

    let service = ForcMcpServer::new();
    let server = service.serve(stdio()).await?;

    info!("MCP server started successfully in STDIO mode");
    server.waiting().await?;
    Ok(())
}

async fn run_sse_server(port: u16) -> Result<()> {
    info!("Starting MCP server in SSE mode on port {port}");

    // Create a simple HTTP endpoint that returns JSON response
    let app = axum::Router::new()
        .route("/sse", axum::routing::get(get_server_info))
        .layer(tower_http::cors::CorsLayer::permissive());

    let listener = tokio::net::TcpListener::bind(("127.0.0.1", port)).await?;
    info!("MCP SSE server listening on http://127.0.0.1:{port}/sse");

    axum::serve(listener, app).await.map_err(Into::into)
}

async fn get_server_info() -> axum::Json<serde_json::Value> {
    let server_info = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "notifications/initialized",
        "params": {
            "protocolVersion": "2024-11-05",
            "capabilities": {
                "tools": {},
                "resources": {}
            },
            "serverInfo": {
                "name": "forc-mcp",
                "version": "0.1.0"
            }
        }
    });

    axum::Json(server_info)
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let cli = Cli::parse();
    match cli.command {
        Commands::Stdio => run_stdio_server().await,
        Commands::Sse { port } => run_sse_server(port).await,
    }
}
