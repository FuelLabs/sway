use rmcp::{model::*, service::RequestContext, Error as McpError, RoleServer};

// Resource URI constants
pub const TYPE_ENCODING_REFERENCE_URI: &str = "forc-call://type-encoding-reference";
pub const COMMON_COMMANDS_URI: &str = "forc-call://examples/common-commands";
pub const CONTRACT_SAMPLES_URI: &str = "forc-call://examples/contract-samples";

/// Get the type encoding reference content
pub fn get_type_encoding_reference() -> &'static str {
    r#"# MCP Tool Type Encoding Reference

When calling contract functions through the MCP `call_contract` tool, function arguments must be encoded according to their Sway types in the `function_args` parameter.

## Basic Types

| Type | Example Input | Notes |
|------|---------------|-------|
| `bool` | `true` or `false` | |
| `u8`, `u16`, `u32`, `u64` | `42` | Decimal integers |
| `u128`, `u256` | `340282366920938463463374607431768211455` | Large integers |
| `b256` | `0x0000000000000000000000000000000000000000000000000000000000000042` | 64 hex chars, 0x prefix optional |

## String Types

| Type | Example Input | Notes |
|------|---------------|-------|
| `str` | `"hello"` | Variable-length string |
| `str[N]` | `"abc"` | Fixed-size string array |
| `String` | `"hello world"` | Heap-allocated string |

## Collection Types

| Type | Example Input | Notes |
|------|---------------|-------|
| Array `[T; N]` | `[1, 2, 3]` | Fixed-size, same type |
| Vector `Vec<T>` | `[42, 128, 256]` | Dynamic-size, same type |
| Tuple `(T1, T2, ...)` | `(42, true, "hello")` | Mixed types allowed |

## Complex Types

### Structs
Structs are encoded as tuples of their fields in declaration order:
```sway
struct Point { x: u64, y: u64 }
```
Input: `{42, 128}` or `(42, 128)`

### Enums
Enums use variant name or index with value:
```sway
enum Status { 
    Inactive,
    Active(bool),
    Pending(u64)
}
```
Input examples:
- `(Inactive: ())` or `(0: ())`
- `(Active: true)` or `(1: true)`
- `(Pending: 42)` or `(2: 42)`

### Option Type
```sway
Option<T>
```
Input examples:
- None: `(None: ())`
- Some: `(Some: 42)`

## Advanced Types

| Type | Example Input | Notes |
|------|---------------|-------|
| `Bytes` | `0x48656c6c6f` | Hex-encoded bytes |
| `RawSlice` | `0x42` | Raw bytes, 0x prefix optional |
| `Identity` | `0x1234...` | Address or ContractId |
| `Address` | `0x1234...` | 64 hex chars |
| `ContractId` | `0x5678...` | 64 hex chars |
| `AssetId` | `0xabcd...` | 64 hex chars |

## MCP Tool Usage

When using the `call_contract` MCP tool, provide these encoded values in the `function_args` array:

```json
{
  "contract_id": "0x1234...",
  "abi": "{\"functions\":[{\"name\":\"transfer\",\"inputs\":[{\"name\":\"to\",\"type\":\"address\"},{\"name\":\"amount\",\"type\":\"u64\"}]}]}", 
  "function": "transfer",
  "function_args": ["\"0x5678...\"", "1000"]
}
```

## Tips
- Strings must be quoted: `"text"`
- Hex values can omit 0x prefix
- Nested structures follow the same rules recursively
- Use parentheses for tuples and braces or parentheses for structs
- Each argument goes as a separate string in the `function_args` array
"#
}

/// Get the common commands content
pub fn get_common_commands() -> &'static str {
    r#"# Common MCP Tool Usage

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
"#
}

/// Get the contract samples content
pub fn get_contract_samples() -> &'static str {
    r#"# Contract Examples with MCP Tool Usage

## 1. Simple Counter Contract

### Contract Code
```sway
contract;

storage {
    count: u64 = 0,
}

abi Counter {
    #[storage(read, write)]
    fn increment();
    
    #[storage(read)]
    fn get_count() -> u64;
    
    #[storage(write)]
    fn set_count(value: u64);
}

impl Counter for Contract {
    #[storage(read, write)]
    fn increment() {
        storage.count.write(storage.count.read() + 1);
    }
    
    #[storage(read)]
    fn get_count() -> u64 {
        storage.count.read()
    }
    
    #[storage(write)]
    fn set_count(value: u64) {
        storage.count.write(value);
    }
}
```

### MCP Tool Commands
```json
// Increment the counter
{
  "tool": "call_contract",
  "parameters": {
    "contract_id": "0x1234...",
    "abi": "{\"functions\":[{\"name\":\"increment\",\"attributes\":[\"storage(read, write)\"]}]}",
    "function": "increment",
    "mode": "live",
    "signing_key": "your-private-key"
  }
}

// Get current count
{
  "tool": "call_contract",
  "parameters": {
    "contract_id": "0x1234...",
    "abi": "{\"functions\":[{\"name\":\"get_count\",\"outputs\":[{\"type\":\"u64\"}],\"attributes\":[\"storage(read)\"]}]}", 
    "function": "get_count"
  }
}

// Set count to specific value
{
  "tool": "call_contract",
  "parameters": {
    "contract_id": "0x1234...",
    "abi": "https://raw.githubusercontent.com/FuelLabs/sway-applications/main/counter/out/debug/counter-abi.json",
    "function": "set_count",
    "function_args": ["42"],
    "mode": "live",
    "signing_key": "your-private-key"
  }
}
```

## 2. Token Contract

### Contract Code
```sway
contract;

use std::auth::msg_sender;

storage {
    balances: StorageMap<Identity, u64> = StorageMap {},
    total_supply: u64 = 0,
}

abi Token {
    #[storage(read, write)]
    fn mint(recipient: Identity, amount: u64);
    
    #[storage(read, write)]
    fn transfer(to: Identity, amount: u64);
    
    #[storage(read)]
    fn balance_of(account: Identity) -> u64;
}

impl Token for Contract {
    #[storage(read, write)]
    fn mint(recipient: Identity, amount: u64) {
        let current = storage.balances.get(recipient).try_read().unwrap_or(0);
        storage.balances.insert(recipient, current + amount);
        storage.total_supply.write(storage.total_supply.read() + amount);
    }
    
    #[storage(read, write)]
    fn transfer(to: Identity, amount: u64) {
        let sender = msg_sender().unwrap();
        let sender_balance = storage.balances.get(sender).read();
        require(sender_balance >= amount, "Insufficient balance");
        
        storage.balances.insert(sender, sender_balance - amount);
        let recipient_balance = storage.balances.get(to).try_read().unwrap_or(0);
        storage.balances.insert(to, recipient_balance + amount);
    }
    
    #[storage(read)]
    fn balance_of(account: Identity) -> u64 {
        storage.balances.get(account).try_read().unwrap_or(0)
    }
}
```

### MCP Tool Commands
```json
// Mint tokens (Identity can be Address or ContractId)
{
  "tool": "call_contract",
  "parameters": {
    "contract_id": "0x1234...",
    "abi": "https://raw.githubusercontent.com/FuelLabs/sway-applications/main/native-asset/out/debug/native-asset-abi.json",
    "function": "mint",
    "function_args": ["\"0x5678...\"", "1000000"],
    "mode": "live",
    "signing_key": "your-private-key"
  }
}

// Transfer tokens
{
  "tool": "call_contract",
  "parameters": {
    "contract_id": "0x1234...",
    "abi": "{\"functions\":[{\"name\":\"transfer\",\"inputs\":[{\"name\":\"to\",\"type\":\"Identity\"},{\"name\":\"amount\",\"type\":\"u64\"}]}]}",
    "function": "transfer", 
    "function_args": ["\"0x9abc...\"", "5000"],
    "mode": "live",
    "signing_key": "your-private-key"
  }
}

// Check balance
{
  "tool": "call_contract",
  "parameters": {
    "contract_id": "0x1234...",
    "abi": "{\"functions\":[{\"name\":\"balance_of\",\"inputs\":[{\"name\":\"account\",\"type\":\"Identity\"}],\"outputs\":[{\"type\":\"u64\"}]}]}",
    "function": "balance_of",
    "function_args": ["\"0x5678...\""]
  }
}
```

## 3. Complex Types Contract

### Contract Code
```sway
contract;

struct User {
    name: str[32],
    age: u64,
    active: bool,
}

enum OrderStatus {
    Pending: (),
    Processing: u64,  // timestamp
    Completed: (u64, b256),  // timestamp, tx_hash
}

abi ComplexTypes {
    fn create_user(user: User) -> User;
    fn process_order(order_id: u64, status: OrderStatus);
    fn batch_operation(users: Vec<User>, amounts: [u64; 3]);
}

impl ComplexTypes for Contract {
    fn create_user(user: User) -> User {
        user
    }
    
    fn process_order(order_id: u64, status: OrderStatus) {
        // Process based on status
    }
    
    fn batch_operation(users: Vec<User>, amounts: [u64; 3]) {
        // Batch processing logic
    }
}
```

### MCP Tool Commands
```json
// Create user (struct as tuple)
{
  "tool": "call_contract",
  "parameters": {
    "contract_id": "0x1234...",
    "abi": "{\"types\":[{\"typeId\":0,\"type\":\"struct User\",\"components\":[{\"name\":\"name\",\"type\":\"str[32]\"},{\"name\":\"age\",\"type\":\"u64\"},{\"name\":\"active\",\"type\":\"bool\"}]}],\"functions\":[{\"name\":\"create_user\",\"inputs\":[{\"name\":\"user\",\"type\":0}],\"outputs\":[{\"type\":0}]}]}",
    "function": "create_user",
    "function_args": ["\"{\\\"Alice\\\", 25, true}\""]
  }
}

// Update order status to Processing  
{
  "tool": "call_contract",
  "parameters": {
    "contract_id": "0x1234...",
    "abi": "https://api.example.com/contracts/order-processor/abi.json",
    "function": "process_order",
    "function_args": ["12345", "\"(Processing: 1699564800)\""],
    "mode": "live",
    "signing_key": "your-private-key"
  }
}

// Update order status to Completed
{
  "tool": "call_contract",
  "parameters": {
    "contract_id": "0x1234...",
    "abi": "https://api.example.com/contracts/order-processor/abi.json",
    "function": "process_order",
    "function_args": ["12345", "\"(Completed: (1699651200, 0x1234567890abcdef...))\""],
    "mode": "live",
    "signing_key": "your-private-key"
  }
}

// Batch operation with vector and array
{
  "tool": "call_contract", 
  "parameters": {
    "contract_id": "0x1234...",
    "abi": "{\"functions\":[{\"name\":\"batch_operation\",\"inputs\":[{\"name\":\"users\",\"type\":\"Vec<User>\"},{\"name\":\"amounts\",\"type\":\"[u64; 3]\"}]}]}",
    "function": "batch_operation",
    "function_args": [
      "\"[{\\\"Alice\\\", 25, true}, {\\\"Bob\\\", 30, true}]\"",
      "\"[100, 200, 300]\""
    ],
    "mode": "live",
    "signing_key": "your-private-key"
  }
}
```

## 4. Multi-Contract Interaction

### Contract Code
```sway
contract;

use std::call_frames::msg_asset_id;

abi Exchange {
    fn swap(
        token_in: ContractId,
        token_out: ContractId,
        amount_in: u64,
        min_amount_out: u64,
    );
}

impl Exchange for Contract {
    fn swap(
        token_in: ContractId,
        token_out: ContractId,
        amount_in: u64,
        min_amount_out: u64,
    ) {
        // Swap logic with external token contracts
    }
}
```

### MCP Tool Commands with Tracing
```json
// Swap tokens and get detailed trace
{
  "tool": "call_contract",
  "parameters": {
    "contract_id": "0xexchange...",
    "abi": "https://raw.githubusercontent.com/FuelLabs/sway-applications/main/AMM/project/contracts/exchange-contract/out/debug/exchange-contract-abi.json",
    "function": "swap",
    "function_args": [
      "\"0xtoken1...\"",
      "\"0xtoken2...\"", 
      "1000",
      "950"
    ],
    "mode": "live",
    "signing_key": "your-private-key",
    "verbosity": 2
  }
}

// Then format the execution trace from the result
{
  "tool": "get_execution_trace",
  "parameters": {
    "trace_events": [/* trace events from call result */],
    "total_gas": 50000,
    "labels": {
      "0xexchange...": "DEX",
      "0xtoken1...": "TokenA", 
      "0xtoken2...": "TokenB"
    }
  }
}
```

## 5. Payable Functions

### Contract Code
```sway
contract;

use std::{
    asset::transfer,
    call_frames::{msg_asset_id, msg_amount},
    context::msg_sender,
};

abi Vault {
    #[payable]
    fn deposit();
    
    #[payable]
    fn deposit_with_data(note: str[64]);
    
    fn withdraw(amount: u64, asset_id: AssetId);
}

impl Vault for Contract {
    #[payable]
    fn deposit() {
        // Automatically receives the asset
    }
    
    #[payable]
    fn deposit_with_data(note: str[64]) {
        let depositor = msg_sender().unwrap();
        let asset = msg_asset_id();
        let amount = msg_amount();
        // Process deposit with metadata
    }
    
    fn withdraw(amount: u64, asset_id: AssetId) {
        let sender = msg_sender().unwrap();
        transfer(sender, asset_id, amount);
    }
}
```

### MCP Tool Commands
```json
// Deposit native asset
{
  "tool": "call_contract",
  "parameters": {
    "contract_id": "0xvault...",
    "abi": "{\"functions\":[{\"name\":\"deposit\",\"attributes\":[\"payable\"]}]}",
    "function": "deposit",
    "amount": 10000,
    "mode": "live",
    "signing_key": "your-private-key"
  }
}

// Deposit with note
{
  "tool": "call_contract",
  "parameters": {
    "contract_id": "0xvault...",
    "abi": "https://api.example.com/contracts/vault/abi.json",
    "function": "deposit_with_data",
    "function_args": ["\"Savings for vacation\""],
    "amount": 5000,
    "mode": "live",
    "signing_key": "your-private-key"
  }
}

// Deposit custom asset
{
  "tool": "call_contract",
  "parameters": {
    "contract_id": "0xvault...",
    "abi": "{\"functions\":[{\"name\":\"deposit\",\"attributes\":[\"payable\"]}]}",
    "function": "deposit",
    "amount": 1000,
    "asset_id": "0xtoken...",
    "mode": "live",
    "signing_key": "your-private-key"
  }
}

// Withdraw
{
  "tool": "call_contract",
  "parameters": {
    "contract_id": "0xvault...",
    "abi": "{\"functions\":[{\"name\":\"withdraw\",\"inputs\":[{\"name\":\"amount\",\"type\":\"u64\"},{\"name\":\"asset_id\",\"type\":\"AssetId\"}]}]}",
    "function": "withdraw",
    "function_args": ["2500", "\"0xasset...\""],
    "mode": "live",
    "signing_key": "your-private-key"
  }
}
```

## Tips for MCP Tool Usage

1. **ABI Sources**: Use inline JSON ABI or remote URLs - relative file paths won't work in MCP
2. **Inline ABI (recommended)**: Provide ABI as escaped JSON string for better reliability
3. **Remote ABI URLs**: Use GitHub raw URLs or hosted API endpoints for ABI files
4. **Use quotes for strings**: Always quote string arguments in `function_args`
5. **Hex format for bytes32**: Use full 64-character hex for b256, Address, ContractId
6. **Tuple notation**: Use parentheses for tuples: `(value1, value2)`
7. **Struct as tuple**: Structs are encoded as tuples in order of fields
8. **Enum variants**: Use variant name with value: `(VariantName: value)`
9. **Arrays vs Vectors**: Both use square brackets but arrays have fixed size
10. **Gas settings**: Use `gas_price` parameter for gas control
11. **Separate arguments**: Each function argument goes as a separate string in `function_args` array
12. **Mode selection**: Use `dry-run` (default), `simulate`, or `live` modes
13. **Asset transfers**: Use `amount` and `asset_id` parameters for payable functions
"#
}

/// Handle resource read requests
pub async fn read_resource(
    uri: &str,
    _: RequestContext<RoleServer>,
) -> Result<ReadResourceResult, McpError> {
    match uri {
        TYPE_ENCODING_REFERENCE_URI => {
            let content = get_type_encoding_reference();
            Ok(ReadResourceResult {
                contents: vec![ResourceContents::text(content, uri)],
            })
        }
        COMMON_COMMANDS_URI => {
            let content = get_common_commands();
            Ok(ReadResourceResult {
                contents: vec![ResourceContents::text(content, uri)],
            })
        }
        CONTRACT_SAMPLES_URI => {
            let content = get_contract_samples();
            Ok(ReadResourceResult {
                contents: vec![ResourceContents::text(content, uri)],
            })
        }
        _ => Err(McpError::resource_not_found(
            "Resource not found",
            Some(serde_json::json!({
                "uri": uri
            })),
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::{COMMON_COMMANDS_URI, CONTRACT_SAMPLES_URI, TYPE_ENCODING_REFERENCE_URI};
    use crate::tests::ForcMcpClient;
    use anyhow::Result;

    #[tokio::test]
    async fn test_forc_call_resources() -> Result<()> {
        let mut client = ForcMcpClient::http_stream_client().await?;

        // List resources
        let resources = client.list_resources().await?;
        assert_eq!(resources.len(), 3);
        assert!(resources.contains(&TYPE_ENCODING_REFERENCE_URI.to_string()));
        assert!(resources.contains(&COMMON_COMMANDS_URI.to_string()));
        assert!(resources.contains(&CONTRACT_SAMPLES_URI.to_string()));

        // Read type encoding reference
        let type_ref = client.read_resource(TYPE_ENCODING_REFERENCE_URI).await?;
        assert!(type_ref.contains("MCP Tool Type Encoding Reference"));
        assert!(type_ref.contains("bool"));
        assert!(type_ref.contains("`u8`, `u16`, `u32`, `u64`"));
        assert!(type_ref.contains("Structs are encoded as tuples"));
        assert!(type_ref.contains("call_contract"));

        // Read common commands
        let commands = client.read_resource(COMMON_COMMANDS_URI).await?;
        assert!(commands.contains("Common MCP Tool Usage"));
        assert!(commands.contains("\"mode\": \"dry-run\""));
        assert!(commands.contains("\"mode\": \"simulate\""));
        assert!(commands.contains("\"mode\": \"live\""));
        assert!(commands.contains("\"tool\": \"call_contract\""));

        // Read contract samples
        let samples = client.read_resource(CONTRACT_SAMPLES_URI).await?;
        assert!(samples.contains("Contract Examples with MCP Tool Usage"));
        assert!(samples.contains("Simple Counter Contract"));
        assert!(samples.contains("Token Contract"));
        assert!(samples.contains("Complex Types Contract"));
        assert!(samples.contains("MCP Tool Commands"));

        Ok(())
    }

    #[tokio::test]
    async fn test_resource_not_found() -> Result<()> {
        let mut client = ForcMcpClient::http_stream_client().await?;

        // Try to read non-existent resource
        let result = client.read_resource("forc-call://non-existent").await;
        assert!(result.is_err());

        Ok(())
    }
}
