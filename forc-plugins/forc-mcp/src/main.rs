use anyhow::Result;
use clap::{Parser, Subcommand};
use forc_mcp::{
    forc_call::ForcCallTools, run_http_server, run_sse_server, run_stdio_server, ForcMcpServer,
};

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
        #[arg(short, long, default_value = "3001")]
        port: u16,
    },
    /// Run MCP server in HTTP streamable mode
    Http {
        /// Port to bind the HTTP server to
        #[arg(short, long, default_value = "3001")]
        port: u16,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("forc_mcp=info".parse().unwrap()),
        )
        .init();

    // Create the MCP server and register tool modules
    let mcp_server = ForcMcpServer::new().register_module(ForcCallTools::new());

    let cli = Cli::parse();
    match cli.command {
        Commands::Stdio => run_stdio_server(mcp_server).await,
        Commands::Sse { port } => run_sse_server(mcp_server, Some(port)).await,
        Commands::Http { port } => run_http_server(mcp_server, Some(port)).await,
    }
}
