# forc mcp

`forc-mcp` is a modular Model Context Protocol (MCP) server that provides AI assistants and tools with programmatic access to the Sway/Fuel ecosystem.  
It exposes functionality from various Forc tools through a standardized MCP interface, enabling seamless integration with AI agents like Claude.

## Overview

The Model Context Protocol (MCP) is a standard for connecting AI assistants to external tools and data sources. `forc-mcp` implements this protocol using a modular architecture that can expose multiple Forc tools and utilities.

### Modular Architecture

`forc-mcp` is designed with extensibility in mind. The server uses a plugin-based architecture where each Forc tool can be wrapped as an MCP module. This allows for:

- **Easy integration** of new Forc tools as MCP modules
- **Isolated functionality** - each module handles its specific domain
- **Consistent interface** - all modules follow the same MCP protocol
- **Selective loading** - choose which modules to enable

## Installation

Build and install `forc-mcp` from the Sway repository:

```bash
# Install from source
cargo install --path ./forc-plugins/forc-mcp

# Or install all forc tools
cargo install --path ./forc
```

## Server Modes

`forc-mcp` supports three transport modes for different integration scenarios:

### STDIO Mode

Standard input/output communication for direct process integration:

```bash
forc-mcp stdio
```

Most efficient for single-session interactions and direct integration with applications.

### Server-Sent Events (SSE) Mode

HTTP server with real-time event streaming:

```bash
forc-mcp sse --port 3001
```

Endpoints:

- `/sse` - Event stream endpoint
- `/message` - Message sending endpoint

Suitable for web-based integrations requiring real-time updates.

### HTTP Streamable Mode

Full HTTP-based MCP protocol implementation:

```bash
forc-mcp http --port 3001
```

Endpoints:

- `/mcp` - Main MCP protocol endpoint

Most suitable for distributed systems and web applications.

## IDE and CLI Integration

### Claude Code CLI

To integrate `forc-mcp` with Claude Code CLI, add the MCP server using one of these transport methods:

```bash
# STDIO transport (requires building forc-mcp first)
claude mcp add --transport stdio forc-mcp-stdio ./target/debug/forc-mcp stdio

# HTTP transport
claude mcp add --transport http forc-mcp-http http://localhost:3001/mcp

# HTTP transport with API key authentication
claude mcp add --transport http forc-mcp-http http://localhost:3001/mcp -H "X-Api-Key: mcp_XXXXXX..."

# Server-Sent Events transport
claude mcp add --transport sse forc-mcp-sse http://localhost:3001/sse
```

### Cursor IDE

For Cursor IDE integration, add the following configuration to your MCP settings file:

#### STDIO Transport

```json
{
  "mcpServers": {
    "forc-mcp": {
      "command": "./target/debug/forc-mcp",
      "args": ["stdio"],
      "transport": "stdio"
    }
  }
}
```

#### HTTP Transport (streamable HTTP)

```json
{
  "mcpServers": {
    "forc-mcp-http": {
      "url": "http://localhost:3001/mcp",
      "transport": "http"
    }
  }
}
```

#### HTTP Transport with API key authentication

```json
{
  "mcpServers": {
    "forc-mcp-http": {
      "url": "http://localhost:3001/mcp",
      "transport": "http",
      "headers": {
        "X-Api-Key": "mcp_XXXXXX..."
      }
    }
  }
}
```

#### SSE Transport

```json
{
  "mcpServers": {
    "forc-mcp-sse": {
      "url": "http://localhost:3001/sse",
      "transport": "sse"
    }
  }
}
```

## Extensibility

`forc-mcp` is designed to accommodate additional Forc tools as MCP modules. Future modules could include:

### Potential Future Modules

- **forc-deploy Module** - Contract deployment tools
- **forc-test Module** - Test execution and reporting
- **forc-doc Module** - Documentation generation and access
- **forc-fmt Module** - Code formatting tools
- **forc-lsp Module** - Language server integration for development workflows
- **forc-node Module** - Local node management

### Adding New Modules

The modular architecture allows developers to add new modules by:

1. **Implementing the `McpToolModule` trait** for the new tool
2. **Registering the module** with the main server using `register_module()`
3. **Defining tool schemas** and resource endpoints
4. **Adding documentation resources** specific to the module

Each module operates independently and can expose its own set of tools, resources, and documentation.

For more detailed examples and advanced usage, see the built-in documentation resources accessible through the MCP tools.
