# forc-call mcp

The forc-call module provides MCP tools for interacting with deployed Fuel contracts, enabling AI assistants to call functions, inspect ABIs, transfer assets, and analyze execution traces.

## forc-call mcp tools

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

## forc-call mcp resources

The forc-call module provides comprehensive documentation resources accessible through the MCP protocol:

- **Type Encoding Reference** (`forc-call://type-encoding-reference`)
  - Complete guide for encoding Sway types for function arguments
  - See: [type_encoding_reference.md](../forc_call/type_encoding_reference.md)
  
- **Common Commands** (`forc-call://examples/common-commands`)
  - Examples of typical usage patterns and tool invocations
  - See: [common_commands.md](../forc_call/common_commands.md)
  
- **Contract Samples** (`forc-call://examples/contract-samples`)
  - Real contract examples with MCP tool commands
  - See: [contract_samples.md](../forc_call/contract_samples.md)

### Accessing resources

Resources can be accessed through the MCP resources API:

1. Use `list_resources` to see all available resources
2. Use `read_resource` with the URIs above to access specific documentation
3. Resources are served by the MCP server at runtime for AI assistants

## Execution Modes

The forc-call module supports three execution modes:

### dry-run (default)
- Validates the transaction without executing
- Returns expected outputs without modifying state
- No signing key required
- Useful for testing and validation

### simulate
- Executes the transaction in a simulated environment
- Shows state changes and gas usage
- No signing key required
- Provides detailed execution trace

### live
- Executes the transaction on the blockchain
- Requires a signing key
- Modifies blockchain state permanently
- Returns transaction ID and receipts

## Type Encoding

When calling contract functions, arguments must be encoded according to their Sway types. Refer to the Type Encoding Reference resource for detailed information on encoding various types including:

- Basic types (bool, integers)
- Addresses and ContractId
- Arrays and vectors
- Strings
- Structs and enums
- Complex nested types

## Error Handling

The module provides detailed error messages for common issues:

- Invalid contract addresses
- ABI parsing errors
- Type encoding mismatches
- Network connectivity issues
- Insufficient funds or gas
- Function not found in ABI

## Best Practices

1. Always use `dry-run` mode first to validate calls
2. Check function signatures with `list_contract_functions` before calling
3. Use the type encoding reference for complex arguments
4. Provide descriptive labels in execution traces for better readability
5. Handle errors gracefully and provide meaningful feedback
