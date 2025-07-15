# forc mcp

`forc-mcp` is a modular Model Context Protocol (MCP) server that provides AI assistants and tools with programmatic access to the Sway/Fuel ecosystem. It exposes functionality from various Forc tools through a standardized MCP interface, enabling seamless integration with AI agents like Claude.

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

## Available Tools

### forc-call Module Tools

The forc-call module exposes four main tools for blockchain interaction:

#### call_contract

Call functions on deployed Fuel contracts with full support for complex types and execution modes.

**Parameters:**
- `contract_id` (string) - Contract address to call
- `abi` (string) - Contract ABI (JSON string or URL)
- `function` (string) - Function name to call
- `function_args` (array) - Function arguments as encoded strings
- `mode` (string) - Execution mode: `dry-run` (default), `simulate`, or `live`
- `node_url` (string, optional) - Custom node URL
- `signing_key` (string, optional) - Private key for live transactions
- `amount` (number, optional) - Asset amount for payable functions
- `asset_id` (string, optional) - Asset ID for transfers (default: native asset)
- `gas_price` (number, optional) - Gas price setting
- `verbosity` (number, optional) - Output verbosity level (0-3)

**Example:**
```json
{
  "tool": "call_contract",
  "parameters": {
    "contract_id": "0x1234567890abcdef...",
    "abi": "{\"functions\":[{\"name\":\"get_balance\",\"outputs\":[{\"type\":\"u64\"}]}]}",
    "function": "get_balance",
    "mode": "dry-run"
  }
}
```

#### list_contract_functions

List all callable functions in a contract's ABI with example usage commands.

**Parameters:**
- `contract_id` (string) - Contract address
- `abi` (string) - Contract ABI (JSON string or URL)

**Example:**
```json
{
  "tool": "list_contract_functions", 
  "parameters": {
    "contract_id": "0x1234567890abcdef...",
    "abi": "https://api.fuel.network/contract-abi.json"
  }
}
```

#### transfer_assets

Transfer assets directly to addresses or contracts (live mode only).

**Parameters:**
- `signing_key` (string) - Private key for transaction signing
- `recipient` (string) - Recipient address or contract ID
- `amount` (number) - Transfer amount
- `asset_id` (string, optional) - Asset ID (default: native asset)
- `node_url` (string, optional) - Custom node URL
- `verbosity` (number, optional) - Output verbosity level

**Example:**
```json
{
  "tool": "transfer_assets",
  "parameters": {
    "recipient": "0x5678901234abcdef...",
    "amount": 1000,
    "signing_key": "your-private-key"
  }
}
```

#### get_execution_trace

Generate human-readable execution traces from contract call results.

**Parameters:**
- `trace_events` (array) - Trace events from CallResponse
- `total_gas` (number) - Total gas used
- `labels` (object, optional) - Contract address to name mappings

**Example:**
```json
{
  "tool": "get_execution_trace",
  "parameters": {
    "trace_events": [/* events from call result */],
    "total_gas": 50000,
    "labels": {
      "0x1234...": "MainContract",
      "0x5678...": "TokenContract"
    }
  }
}
```

#### forc-call Module Resources

The forc-call module provides comprehensive documentation resources:

- **Type Encoding Reference** - Complete guide for encoding Sway types
- **Common Commands** - Examples of typical usage patterns
- **Contract Samples** - Real contract examples with MCP tool commands

Access these resources through the MCP resources API or the `list_resources` and `read_resource` tools.

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
