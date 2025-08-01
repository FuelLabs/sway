use anyhow::Result;
use clap::{Parser, Subcommand};
use forc_mcp::{
    auth::AuthConfig, forc_call::ForcCallTools, run_http_server, run_sse_server, run_stdio_server,
    ForcMcpServer,
};

/// Model Context Protocol (MCP) server for Forc
#[derive(Parser)]
#[command(name = "forc-mcp")]
#[command(about = "MCP server plugin for Forc")]
#[command(version = env!("CARGO_PKG_VERSION"))]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Default)]
enum Commands {
    /// Run MCP server in STDIO mode
    #[default]
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

        /// Enable authentication mode with API keys
        #[arg(long)]
        auth: bool,

        /// Require API key for all requests (no public access)
        #[arg(long, requires = "auth")]
        api_keys_only: bool,

        /// Path to persist API keys (default: in-memory only)
        #[arg(long, value_name = "FILE")]
        api_keys_file: Option<String>,

        /// Pre-configured admin API key (if not provided, one will be generated)
        #[arg(long, value_name = "KEY", requires = "auth")]
        admin_api_key: Option<String>,

        /// Public rate limit per minute (unauthenticated requests)
        #[arg(long, default_value = "10")]
        public_rate_limit_per_minute: u32,

        /// Public rate limit per day (unauthenticated requests)
        #[arg(long, default_value = "1000")]
        public_rate_limit_per_day: u32,

        /// API key rate limit per minute
        #[arg(long, default_value = "120")]
        api_key_rate_limit_per_minute: u32,

        /// API key rate limit per day
        #[arg(long, default_value = "10000")]
        api_key_rate_limit_per_day: u32,
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
    match cli.command.unwrap_or_default() {
        Commands::Stdio => run_stdio_server(mcp_server).await,
        Commands::Sse { port } => run_sse_server(mcp_server, Some(port)).await,
        Commands::Http {
            port,
            auth,
            api_keys_only,
            api_keys_file,
            admin_api_key,
            public_rate_limit_per_minute,
            public_rate_limit_per_day,
            api_key_rate_limit_per_minute,
            api_key_rate_limit_per_day,
        } => {
            let auth_config = AuthConfig {
                enabled: auth,
                api_keys_only,
                api_keys_file,
                admin_api_key,
                public_rate_limit_per_minute,
                public_rate_limit_per_day,
                api_key_rate_limit_per_minute,
                api_key_rate_limit_per_day,
            };
            run_http_server(mcp_server, Some(port), auth_config).await
        }
    }
}
