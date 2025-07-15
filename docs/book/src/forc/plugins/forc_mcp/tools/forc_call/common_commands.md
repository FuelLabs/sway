# Common MCP Tool Usage

## Basic Function Calls

### Call a simple function
```json
{
  "tool": "call_contract",
  "parameters": {
    "contract_id": "0x1234...",
    "abi": "{\"functions\":[{\"name\":\"get_balance\",\"outputs\":[{\"type\":\"u64\"}]}]}",
    "function": "get_balance"
  }
}
```

### Call with parameters
```json
{
  "tool": "call_contract", 
  "parameters": {
    "contract_id": "0x1234...",
    "abi": "https://api.example.com/contracts/0x1234.../abi.json",
    "function": "transfer",
    "function_args": ["\"0x5678...\"", "1000"]
  }
}
```

### Call with complex types
```json
// Tuple parameter (inline ABI)
{
  "tool": "call_contract",
  "parameters": {
    "contract_id": "0x1234...",
    "abi": "{\"functions\":[{\"name\":\"process_order\",\"inputs\":[{\"name\":\"order\",\"type\":\"(u64, bool, str)\"}]}]}", 
    "function": "process_order",
    "function_args": ["\"(42, true, \\\"urgent\\\")\""]
  }
}

// Struct parameter (URL ABI)
{
  "tool": "call_contract",
  "parameters": {
    "contract_id": "0x1234...",
    "abi": "https://contracts.fuel.network/abi/user-contract.json",
    "function": "create_user", 
    "function_args": ["\"{\\\"Alice\\\", 25, true}\""]
  }
}

// Array parameter (inline ABI)
{
  "tool": "call_contract",
  "parameters": {
    "contract_id": "0x1234...",
    "abi": "{\"functions\":[{\"name\":\"sum_values\",\"inputs\":[{\"name\":\"values\",\"type\":\"[u64; 5]\"}]}]}",
    "function": "sum_values",
    "function_args": ["\"[1, 2, 3, 4, 5]\""]
  }
}

// Enum parameter (URL ABI)
{
  "tool": "call_contract",
  "parameters": {
    "contract_id": "0x1234...",
    "abi": "https://api.example.com/contract-abi.json", 
    "function": "set_status",
    "function_args": ["\"(Active: true)\""]
  }
}
```

## Execution Modes

### Dry-run (default)
```json
{
  "tool": "call_contract",
  "parameters": {
    "contract_id": "0x1234...",
    "abi": "{\"functions\":[{\"name\":\"test_function\"}]}",
    "function": "test_function",
    "mode": "dry-run"
  }
}
```

### Simulate (estimates gas)
```json
{
  "tool": "call_contract", 
  "parameters": {
    "contract_id": "0x1234...",
    "abi": "https://api.fuel.network/contracts/0x1234.../abi",
    "function": "test_function",
    "mode": "simulate"
  }
}
```

### Live (state changes)
```json
{
  "tool": "call_contract",
  "parameters": {
    "contract_id": "0x1234...",
    "abi": "{\"functions\":[{\"name\":\"test_function\"}]}", 
    "function": "test_function",
    "mode": "live",
    "signing_key": "your-private-key"
  }
}
```

## Payable Functions

### Transfer native asset
```json
{
  "tool": "call_contract",
  "parameters": {
    "contract_id": "0x1234...",
    "abi": "{\"functions\":[{\"name\":\"deposit\",\"attributes\":[\"payable\"]}]}",
    "function": "deposit",
    "amount": 1000,
    "mode": "live"
  }
}
```

### Transfer custom asset
```json
{
  "tool": "call_contract",
  "parameters": {
    "contract_id": "0x1234...",
    "abi": "https://contracts.fuel.network/abi/token-vault.json", 
    "function": "deposit_token",
    "amount": 500,
    "asset_id": "0x5678...",
    "mode": "live"
  }
}
```

## Direct Transfers

### Transfer to address/contract
```json
{
  "tool": "transfer_assets",
  "parameters": {
    "recipient": "0x1234...",
    "amount": 1000,
    "signing_key": "your-private-key"
  }
}
```

## Network Selection

### Local node
```json
{
  "tool": "call_contract",
  "parameters": {
    "contract_id": "0x1234...",
    "abi": "{\"functions\":[{\"name\":\"test\"}]}",
    "function": "test",
    "node_url": "http://127.0.0.1:4000"
  }
}
```

### Custom node URL
```json
{
  "tool": "call_contract", 
  "parameters": {
    "contract_id": "0x1234...",
    "abi": "https://raw.githubusercontent.com/FuelLabs/sway-applications/main/contracts/abi.json",
    "function": "test",
    "node_url": "https://mainnet.fuel.network"
  }
}
```

## List Functions

### Show all callable functions
```json
{
  "tool": "list_contract_functions",
  "parameters": {
    "contract_id": "0x1234...",
    "abi": "https://api.fuel.network/contracts/0x1234.../abi"
  }
}
```

## Using Different ABI Sources

### Remote URL
```json
{
  "tool": "call_contract",
  "parameters": {
    "contract_id": "0x1234...",
    "abi": "https://example.com/contract-abi.json",
    "function": "function_name"
  }
}
```

### Inline JSON (recommended for MCP)
```json
{
  "tool": "call_contract",
  "parameters": {
    "contract_id": "0x1234...",
    "abi": "{\"functions\":[{\"name\":\"get\",\"outputs\":[{\"type\":\"u64\"}]}]}",
    "function": "get"
  }
}
```

### GitHub raw URL
```json
{
  "tool": "call_contract",
  "parameters": {
    "contract_id": "0x1234...",
    "abi": "https://raw.githubusercontent.com/FuelLabs/sway-applications/main/AMM/project/contracts/exchange-contract/out/debug/exchange-contract-abi.json",
    "function": "function_name"
  }
}
```

## Execution Trace

### Get formatted execution trace
```json
{
  "tool": "get_execution_trace",
  "parameters": {
    "trace_events": [/* trace events from call result */],
    "total_gas": 12345,
    "labels": {
      "0x1234...": "MyContract",
      "0x5678...": "TokenContract"
    }
  }
}
```