# Contract Examples with MCP Tool Usage

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

### Token Contract Code

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

### Token MCP Tool Commands

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

### Complex Types Contract Code

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

### Complex Types MCP Tool Commands

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

### Exchange Contract Code

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

### Vault Contract Code

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

### Vault MCP Tool Commands

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
