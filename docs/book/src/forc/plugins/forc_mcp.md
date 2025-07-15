# forc mcp

`forc-mcp` is a modular Model Context Protocol (MCP) server that provides AI assistants and tools with programmatic access to the Sway/Fuel ecosystem. It exposes functionality from various Forc tools through a standardized MCP interface, enabling seamless integration with AI agents like Claude.

## Overview

The Model Context Protocol (MCP) is a standard for connecting AI assistants to external tools and data sources. `forc-mcp` implements this protocol using a modular architecture that can expose multiple Forc tools and utilities.

### Current Modules

#### forc-call Module
The first implemented module exposes `forc call` functionality, allowing AI assistants to:

- Call functions on deployed Fuel contracts
- List available contract functions
- Transfer assets between addresses and contracts  
- Generate formatted execution traces
- Access comprehensive documentation and examples

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

### call_contract

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

### list_contract_functions

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

### transfer_assets

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

### get_execution_trace

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

## Type Encoding

Function arguments must be encoded according to their Sway types. Here are the key encoding rules:

| Type | Example Input | Notes |
|------|---------------|-------|
| `bool` | `true` or `false` | |
| `u8`, `u16`, `u32`, `u64` | `42` | Decimal integers |
| `u128`, `u256` | `340282366920938463463374607431768211455` | Large integers |
| `b256` | `0x0000000000000000000000000000000000000000000000000000000000000042` | 64 hex chars |
| `str` | `"hello"` | Quoted strings |
| Array `[T; N]` | `[1, 2, 3]` | Fixed-size arrays |
| Vector `Vec<T>` | `[42, 128, 256]` | Dynamic arrays |
| Tuple `(T1, T2)` | `(42, true)` | Mixed types |
| Struct | `{42, "name", true}` | Fields in declaration order |
| Enum | `(Active: true)` | Variant name/index with value |

**Important Notes:**
- Each function argument goes as a separate string in the `function_args` array
- Strings must be quoted: `"text"`
- Structs are encoded as tuples of their fields in declaration order
- Enums use variant name or index: `(VariantName: value)` or `(0: value)`

## ABI Sources

Contract ABIs can be provided in multiple ways:

### Inline JSON (Recommended)
```json
{
  "abi": "{\"functions\":[{\"name\":\"get\",\"outputs\":[{\"type\":\"u64\"}]}]}"
}
```

### Remote URLs
```json
{
  "abi": "https://api.fuel.network/contracts/0x1234.../abi.json"
}
```

### GitHub Raw URLs
```json
{
  "abi": "https://raw.githubusercontent.com/FuelLabs/sway-applications/main/counter/out/debug/counter-abi.json"
}
```

## Execution Modes

### Dry-run (Default)
- No state changes
- No gas consumption
- Fast execution for testing

### Simulate
- Validates transaction
- Estimates gas usage
- No state changes

### Live
- Executes on blockchain
- Requires signing key
- State changes persist
- Consumes gas

## Built-in Documentation

### forc-call Module Resources

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

## Network Configuration

By default, `forc-mcp` connects to a local Fuel node. Specify custom networks:

```json
{
  "node_url": "https://mainnet.fuel.network"
}
```

Common networks:
- Local: `http://127.0.0.1:4000`
- Testnet: `https://testnet.fuel.network`
- Mainnet: `https://mainnet.fuel.network`

## Error Handling

All tools return structured error responses instead of throwing exceptions:

```json
{
  "is_error": true,
  "content": [
    {
      "type": "text",
      "text": "Error: Invalid contract address: ..."
    }
  ]
}
```

Common error scenarios:
- Invalid contract addresses or ABIs
- Missing or incorrectly formatted function arguments
- Network connectivity issues
- Insufficient funds for transfers
- Function not found in contract

## Integration Examples

### With Claude

Configure Claude to use the MCP server in your `.claude/claude_desktop_config.json`:

```json
{
  "mcpServers": {
    "forc-mcp": {
      "command": "forc-mcp",
      "args": ["stdio"]
    }
  }
}
```

### With Custom Applications

Connect via HTTP for web applications:

```javascript
// Start HTTP server
// forc-mcp http --port 3001

const response = await fetch('http://localhost:3001/mcp', {
  method: 'POST',
  headers: { 'Content-Type': 'application/json' },
  body: JSON.stringify({
    method: 'tools/call',
    params: {
      name: 'call_contract',
      arguments: {
        contract_id: '0x1234...',
        abi: 'contract-abi.json',
        function: 'get_balance'
      }
    }
  })
});
```

## Best Practices

1. **Use inline ABI JSON** for better reliability over remote URLs
2. **Start with dry-run mode** to test function calls before going live
3. **Provide contract labels** in traces for better readability
4. **Use structured error handling** - check `is_error` field in responses
5. **Quote string arguments** properly in `function_args`
6. **Test complex types** in dry-run mode first
7. **Use appropriate verbosity levels** for debugging (0-3)
8. **Specify gas settings** for production transactions

## Troubleshooting

### Common Issues

**ABI Mismatch:**
- Ensure ABI matches the deployed contract
- Verify function names and parameter types
- Check that the contract is deployed at the specified address

**Parameter Type Errors:**
- Review type encoding guide for correct format
- Ensure correct number of parameters
- Check that arrays/vectors have consistent types

**Network Issues:**
- Verify node URL accessibility
- Check network selection (local/testnet/mainnet)
- Ensure node is synced and running

**Transaction Failures:**
- Use simulate mode to debug before live execution
- Verify wallet has sufficient balance
- Check gas settings and limits

**Resource Access:**
- Use MCP resource URIs: `forc-call://type-encoding-reference`
- Check that documentation resources are accessible
- Verify MCP client supports resource reading

For more detailed examples and advanced usage, see the built-in documentation resources accessible through the MCP tools.