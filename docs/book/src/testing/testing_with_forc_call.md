# Forc Call

`forc-call` is a command-line tool for interacting with deployed Fuel contracts. It allows you to make contract calls, query contract state, and interact with any deployed contract on the Fuel network - all from your command line!

The `forc call` command is part of the Forc toolchain and is installed alongside other Forc tools.

## Getting Started

Here are a few examples of what you can do with `forc call`:

```sway
contract;

abi ContractABI {
  fn add(a: u64, b: u64) -> u64;
}

impl ContractABI for Contract {
  fn add(a: u64, b: u64) -> u64 {
    a + b
  }
}
```

### List callable functions of a contract given it's ABI file

```bash
forc call 0xe18de7c7c8c61a1c706dccb3533caa00ba5c11b5230da4428582abf1b6831b4d \
  --abi ./out/debug/counter-contract-abi.json \
  --list-functions
```

Output:

```log
Available functions in contract: 0xe18de7c7c8c61a1c706dccb3533caa00ba5c11b5230da4428582abf1b6831b4d

add(a: u64, b: u64) -> u64
  forc call \
    --abi ./out/debug/counter-contract-abi.json \
    0xe18de7c7c8c61a1c706dccb3533caa00ba5c11b5230da4428582abf1b6831b4d \
    add "0" "0"
```

### List functions from multiple contracts with additional ABIs

```bash
forc call 0xe18de7c7c8c61a1c706dccb3533caa00ba5c11b5230da4428582abf1b6831b4d \
  --abi ./out/debug/counter-contract-abi.json \
  --contract-abi 0xf8f8b6283d7fa5b672b530cbb84fcccb4ff8dc40f8176ef4544ddb1f1952ad07:./token-abi.json \
  --contract-abi 0x1234567890abcdef:https://example.com/pool-abi.json \
  --list-functions
```

### Call a simple addition function on a deployed contract (in dry-run mode)

```bash
forc call 0xe18de7c7c8c61a1c706dccb3533caa00ba5c11b5230da4428582abf1b6831b4d \
  --abi ./out/debug/counter-contract-abi.json \
  add 1 2
```

### Call a contract with labeled addresses for better trace readability

```bash
forc call 0xe18de7c7c8c61a1c706dccb3533caa00ba5c11b5230da4428582abf1b6831b4d \
  --abi ./out/debug/counter-contract-abi.json \
  transfer 0xf8f8b6283d7fa5b672b530cbb84fcccb4ff8dc40f8176ef4544ddb1f1952ad07 \
  --label 0xe18de7c7c8c61a1c706dccb3533caa00ba5c11b5230da4428582abf1b6831b4d:MainContract \
  --label 0xf8f8b6283d7fa5b672b530cbb84fcccb4ff8dc40f8176ef4544ddb1f1952ad07:TokenContract \
  -vvv
```

### Directly send funds to an address

```bash
forc call 0x2c7Fd852EF2BaE281e90ccaDf18510701989469f7fc4b042F779b58a39919Eec --amount 2 --mode=live
```

### Query the owner of a deployed [DEX contract](https://github.com/mira-amm/mira-v1-core) on testnet

```bash
forc call \
  --testnet \
  --abi https://raw.githubusercontent.com/mira-amm/mira-v1-periphery/refs/heads/main/fixtures/mira-amm/mira_amm_contract-abi.json \
  0xd5a716d967a9137222219657d7877bd8c79c64e1edb5de9f2901c98ebe74da80 \
  owner
```

## Usage

Forc call has **3** usage modes:

### List functions

Syntax for `forc call` for listing supported functions from the ABI - with example command to perform call operation:

```bash
forc call --abi <ABI-PATH/URL> <CONTRACT_ID> --list-functions
```

You can also list functions from multiple contracts by providing additional contract ABIs:

```bash
forc call --abi <MAIN-ABI-PATH/URL> <MAIN-CONTRACT_ID> \
  --contract-abi <CONTRACT_ID>:<ABI-PATH/URL> \
  --contract-abi <ANOTHER-CONTRACT_ID>:<ABI-PATH/URL> \
  --list-functions
```

Where the following arguments are required:

- `ABI-PATH/URL` is the path or URL to the contract's JSON ABI file
- `CONTRACT_ID` is the ID of the deployed contract you want to interact with
- `--contract-abi` (optional) allows you to specify additional contracts and their ABIs to list functions from multiple contracts at once

### Transfer assets

Syntax for `forc call` for transferring assets:

```bash
forc call <RECEIVER_ADDRESS> --amount <AMOUNT> --mode=live
```

Where the following arguments are required:

- `RECEIVER_ADDRESS` is address of the receiver (identity or contract).
- `AMOUNT` is the amount of assets to transfer.

Note: only live mode `--mode=live` is supported; transfers cannot be simulated.

### Call contracts

Syntax for `forc call` for contract calls:

```bash
forc call [OPTIONS] --abi <ABI-PATH/URL> <CONTRACT_ID> <SELECTOR> [ARGS]...
```

Where the following arguments are required:

- `CONTRACT_ID` is the ID of the deployed contract you want to interact with
- `ABI-PATH/URL` is the path or URL to the contract's JSON ABI file
- `SELECTOR` is the function name (selector) you want to call
- `ARGS` are the arguments to pass to the function

## Type Encoding

When passing arguments to contract functions, values are encoded according to their Sway types.
Here's how to format different types:

| Types                                         | Example input                                                        | Notes                                                                                          |
|-----------------------------------------------|----------------------------------------------------------------------|------------------------------------------------------------------------------------------------|
| bool                                          | `true` or `false`                                                    |                                                                                                |
| u8, u16, u32, u64, u128, u256                 | `42`                                                                 |                                                                                                |
| b256                                          | `0x0000000000000000000000000000000000000000000000000000000000000042` or `0000000000000000000000000000000000000000000000000000000000000042` | `0x` prefix is optional |
| bytes, RawSlice                               | `0x42` or `42`                                                       | `0x` prefix is optional                                                                                               |
| String, StringSlice, StringArray (Fixed-size) | `"abc"`                                                              |                                                                                                |
| Tuple                                         | `(42, true)`                                                         | The types in tuple can be different                                                                                               |
| Array (Fixed-size), Vector (Dynamic)          | `[42, 128]`                                                          | The types in array or vector must be the same; i.e. you cannot have `[42, true]`              |
| Struct                                        | `{42, 128}`                                                          | Since structs are packed encoded, the attribute names are not encoded; i.e. `{42, 128}`; this could represent the following `struct Polygon { x: u64, y: u64 }` |
| Enum                                          | `(Active: true)` or `(1: true)`       | Enums are key-val pairs with keys as being variant name (case-sensitive) or variant index (starting from 0) and values as being the variant value; this could represent the following `enum MyEnum { Inactive, Active(bool) }` |

## ABI Support

The ABI (Application Binary Interface) can be provided in two ways.

### Local file

```bash
forc call <CONTRACT_ID> --abi ./path/to/abi.json <FUNCTION> [ARGS...]
```

### Remote ABI file/URL

```bash
forc call <CONTRACT_ID> --abi https://example.com/abi.json <FUNCTION> [ARGS...]
```

## Network Configuration

```bash
forc call --node-url http://127.0.0.1:4000 ...
# or
forc call --target local ...
```

## Advanced Usage

### Using Wallets

```sh
# utilising the forc-wallet
forc call <CONTRACT_ID> --abi <PATH> <FUNCTION> --wallet
```

```sh
# with an explicit signing key
forc call <CONTRACT_ID> --abi <PATH> <FUNCTION> --signing-key <KEY>
```

### Asset Transfers

```sh
# Native asset transfer
forc call <CONTRACT_ID> --abi <PATH> <FUNCTION> --amount 100 --live
```

```sh
# Custom asset transfer
forc call <CONTRACT_ID> --abi <PATH> <FUNCTION> \
    --amount 100 \
    --asset-id 0x1234... \
    --live
```

### Gas Configuration

```sh
# Set gas price
forc call <CONTRACT_ID> --abi <PATH> <FUNCTION> --gas-price 1

# Forward gas to contract
forc call <CONTRACT_ID> --abi <PATH> <FUNCTION> --gas-forwarded 1000

# Set maximum fee
forc call <CONTRACT_ID> --abi <PATH> <FUNCTION> --max-fee 5000
```

### Transaction Tracing

When you need to debug contract interactions or understand the execution flow, `forc call` provides detailed transaction traces with verbosity level 2 or higher (`-vv` or `-v=2`).

```sh
# Enable transaction tracing
forc call <CONTRACT_ID> --abi <PATH> <FUNCTION> -vv
```

The transaction trace provides a hierarchical view of all contract calls, showing:

- Gas consumption for each call (`[gas_amount]`)
- Contract addresses being called
- Return values and data
- Emitted logs and events
- Nested contract calls with proper indentation
- Overall transaction result and gas usage

#### Enhancing Traces with Labels

For better readability, you can label contract addresses in transaction traces using the `--label` flag:

```sh
# Add human-readable labels to contract addresses
forc call <CONTRACT_ID> \
  --abi <PATH> \
  <FUNCTION> \
  --label <CONTRACT_ID>:MainContract \
  --label <OTHER_CONTRACT_ID>:TokenContract \
  -vv
```

**Without labels:**
```log
├─ [8793] 0x2af09151f8276611ba65f14650970657bc42c1503d6502ffbb4d085ec37065dd::transfer(100, 0x123)
│    └─ ← ()
```

**With labels:**
```log
├─ [8793] TokenContract::transfer(100, 0x123)
│    └─ ← ()
```

#### Improving Trace Decoding with Additional ABIs

For complex multi-contract interactions, you can provide additional contract ABIs to improve function signature and return data decoding:

```sh
# Add additional contract ABIs for better trace decoding
forc call <CONTRACT_ID> \
  --abi <MAIN_ABI_PATH> \
  <FUNCTION> \
  --contract-abi <OTHER_CONTRACT_ID>:./external-abi.json \
  --contract-abi <THIRD_CONTRACT_ID>:https://example.com/abi.json \
  -vv
```

This helps decode:
- Function names
- Function parameters in readable format
- Return values in structured format instead of raw hex

#### Example Transaction Trace Output

```bash
forc call 0x9275a76531bce733cfafdbcb6727ea533ebbdc358d685152169b3c4eaa47b965 \
  --abi ./demo/demo-caller-abi.json \
  call_increment_count \
  --label 0x9275a76531bce733cfafdbcb6727ea533ebbdc358d685152169b3c4eaa47b965:DemoCaller \
  --label 0xb792b1e233a2c06bccec611711acc3bb61bdcb28f16abdde86d1478ee02f6e42:Counter \
  --contract-abi 0xb792b1e233a2c06bccec611711acc3bb61bdcb28f16abdde86d1478ee02f6e42:./counter-abi.json \
  -vv
```

**Output:**

```log
Traces:
  [Script]
    ├─ [124116] DemoCaller::call_increment_count()
    │    ├─ [111500] Counter::increment()
    │    │    └─ ← ()
    │    ├─ emit AsciiString { data: "incremented count" }
    │    ├─ [86284] Counter::get_count()
    │    │    └─ ← 0x0000000000000002
    │    ├─ emit 2
    │    ├─ emit AsciiString { data: "done" }
    │    ├─ [72699] Counter::increment()
    │    │    └─ ← ()
    │    ├─ [48287] Counter::get_count()
    │    │    └─ ← 0x0000000000000003
    │    └─ ← 0x0000000000000003
    └─ ← [Return] val: 1
  [ScriptResult] result: Success, gas_used: 89279

Transaction successfully executed.
Gas used: 160676
```

#### Understanding the Trace Format

- `[Script]` - The root transaction script
- `├─ [gas_amount] ContractLabel::function_name(args)` - A contract call with gas consumption and decoded function info
- `│    └─ ← value` - Return value from the contract call
- `emit data` - Log/event emitted by the contract
- Indentation shows the call hierarchy (nested calls are indented further)
- `[ScriptResult]` - Final transaction result with gas used by the script
- `Gas used: <gas_used>` - Total gas used by the transaction

This tracing feature is particularly useful for:

- Debugging failed transactions
- Understanding gas consumption patterns
- Analyzing complex multi-contract interactions
- Verifying expected contract behavior

### Common Use Cases

#### Contract State Queries

```sh
# Read contract state
forc call <CONTRACT_ID> --abi <PATH> get_balance

# Query with parameters
forc call <CONTRACT_ID> --abi <PATH> get_user_info 0x1234...

# Update contract state
forc call <CONTRACT_ID> --abi <PATH> update_state 42 --live
```

#### Token Operations

```sh
# Transfer assets/tokens to an address
forc call <ADDRESS> --amount 100 --live
```

#### Multi-Contract Debugging

```sh
# Debug complex multi-contract interactions
forc call <MAIN_CONTRACT_ID> \
  --abi <MAIN_ABI_PATH> \
  complex_operation \
  --label <MAIN_CONTRACT_ID>:MainContract \
  --label <TOKEN_CONTRACT_ID>:TokenContract \
  --label <POOL_CONTRACT_ID>:LiquidityPool \
  --contract-abi <TOKEN_CONTRACT_ID>:./token-abi.json \
  --contract-abi <POOL_CONTRACT_ID>:./pool-abi.json \
  -vv
```

### CLI Parameters Reference

#### Trace Enhancement Options

- `--label <contract_id>:<label>` - Add human-readable labels for contract addresses in traces
  - Can be used multiple times
  - Contract ID can include or omit `0x` prefix
  - Example: `--label 0x123:MainContract`

- `--contract-abi <contract_id>:<abi_path>` - Specify additional contract ABIs for better trace decoding
  - Can be used multiple times
  - Supports both local files and URLs
  - Contract ID can include or omit `0x` prefix
  - Example: `--contract-abi 0x123:./abi.json` or `--contract-abi 0x456:https://example.com/abi.json`

- `-v, -vv, -vvv` - Verbosity levels for trace output
  - `-v`: Show decoded logs
  - `-vv`: Additionally show transaction traces
  - `-vvv`: Additionally show receipts and script JSON

## Tips and Tricks

- Use `--mode simulate` to estimate gas costs before making live transactions
- External contracts are automatically detected (via internal simulations), but can be manually specified with `--external-contracts`
- For complex parameter types (tuples, structs, enums), refer to the parameter types table above
- Always verify contract addresses and ABIs before making live calls
- Use environment variables for sensitive data like signing keys: `SIGNING_KEY=<key>`
- **Use labels for better trace readability**: Add `--label` flags to replace long contract addresses with meaningful names in traces
- **Provide additional ABIs for multi-contract calls**: Use `--contract-abi` to decode function calls and return values from external contracts
- **Combine labels and ABIs**: Use both together for the most readable and informative transaction traces

## Troubleshooting

### Common issues and solutions

- **ABI Mismatch**:
  - Ensure the ABI matches the deployed contract
  - Verify function selectors match exactly
  - Check that additional contract ABIs match their respective contracts

- **Parameter Type Errors**:
  - Check parameter formats in the types table
  - Ensure correct number of parameters

- **Network Issues**:
  - Verify node connection
  - Check network selection (testnet/mainnet)

- **Transaction Failures**:
  - Use simulation mode to debug
  - Check gas settings
  - Verify wallet has sufficient balance

- **Trace Decoding Issues**:
  - Ensure contract ABIs are correctly specified
  - Verify contract addresses in labels match deployed contracts
  - Check that ABI files are accessible (for both local files and URLs)
