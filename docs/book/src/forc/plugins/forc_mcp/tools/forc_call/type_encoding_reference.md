# MCP Tool Type Encoding Reference

When calling contract functions through the MCP `call_contract` tool, function arguments must be encoded according to their Sway types in the `function_args` parameter.

## Type Encoding Table

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

## Detailed Type Examples

### Basic Types

```json
// Boolean
"function_args": ["true"]

// Integers
"function_args": ["42", "100", "1000000"]

// Large integers (u128, u256)
"function_args": ["340282366920938463463374607431768211455"]

// b256 (both formats work)
"function_args": ["0x0000000000000000000000000000000000000000000000000000000000000042"]
"function_args": ["0000000000000000000000000000000000000000000000000000000000000042"]
```

### String Types

```json
// Variable-length strings
"function_args": ["\"hello world\""]

// Fixed-size string arrays
"function_args": ["\"abc\""]
```

### Collection Types

```json
// Fixed-size arrays
"function_args": ["[1, 2, 3, 4, 5]"]

// Dynamic vectors
"function_args": ["[42, 128, 256]"]

// Tuples (mixed types)
"function_args": ["(42, true, \"hello\")"]
```

### Complex Types

#### Structs

Structs are encoded as tuples of their fields in declaration order:

```sway
struct Point { x: u64, y: u64 }
struct User { name: str[32], age: u64, active: bool }
```

```json
// Point struct
"function_args": ["{42, 128}"]

// User struct  
"function_args": ["{\"Alice\", 25, true}"]
```

#### Enums

Enums use variant name (case-sensitive) or index with value:

```sway
enum Status { 
    Inactive,
    Active(bool),
    Pending(u64)
}
```

```json
// Using variant names
"function_args": ["(Inactive: ())"]
"function_args": ["(Active: true)"] 
"function_args": ["(Pending: 42)"]

// Using variant indices (starting from 0)
"function_args": ["(0: ())"]
"function_args": ["(1: true)"]
"function_args": ["(2: 42)"]
```

#### Option Type

```json
// None variant
"function_args": ["(None: ())"]

// Some variant  
"function_args": ["(Some: 42)"]
```

### Advanced Types

```json
// Bytes (hex-encoded)
"function_args": ["0x48656c6c6f"]

// RawSlice (0x prefix optional)
"function_args": ["0x42"]
"function_args": ["42"]

// Identity (Address or ContractId)
"function_args": ["\"0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef\""]

// Address (64 hex characters)
"function_args": ["\"0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef\""]

// ContractId (64 hex characters)
"function_args": ["\"0x5678901234abcdef5678901234abcdef5678901234abcdef5678901234abcdef\""]

// AssetId (64 hex characters)
"function_args": ["\"0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890\""]
```

## MCP Tool Usage

When using the `call_contract` MCP tool, provide these encoded values in the `function_args` array:

```json
{
  "tool": "call_contract",
  "parameters": {
    "contract_id": "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef",
    "abi": "{\"functions\":[{\"name\":\"transfer\",\"inputs\":[{\"name\":\"to\",\"type\":\"address\"},{\"name\":\"amount\",\"type\":\"u64\"}]}]}", 
    "function": "transfer",
    "function_args": ["\"0x5678901234abcdef5678901234abcdef5678901234abcdef5678901234abcdef\"", "1000"]
  }
}
```

### Complex Example

```json
{
  "tool": "call_contract", 
  "parameters": {
    "contract_id": "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef",
    "abi": "{\"functions\":[{\"name\":\"process_data\",\"inputs\":[{\"name\":\"user\",\"type\":\"struct User\"},{\"name\":\"status\",\"type\":\"enum Status\"},{\"name\":\"amounts\",\"type\":\"[u64; 3]\"}]}]}",
    "function": "process_data",
    "function_args": [
      "{\"Alice\", 25, true}",
      "(Active: true)", 
      "[100, 200, 300]"
    ]
  }
}
```

## Important Notes

- **Strings must be quoted**: Always use double quotes around string values: `"text"`
- **Hex values flexible**: The `0x` prefix is optional for most hex values
- **Nested structures**: Follow the same encoding rules recursively for nested types
- **Struct encoding**: Use braces `{}` or parentheses `()` for structs - fields are encoded in declaration order without names
- **Enum encoding**: Use variant name (case-sensitive) or index with value: `(VariantName: value)`
- **Separate arguments**: Each function argument goes as a separate string in the `function_args` array
- **Array vs Vector**: Both use square brackets but arrays have fixed size, vectors are dynamic
- **Tuple notation**: Use parentheses `()` for tuples, mixed types allowed

## Tips for Success

1. **Test with dry-run first**: Use `"mode": "dry-run"` (default) to test encoding before live calls
2. **Check ABI compatibility**: Ensure your encoded arguments match the function signature in the ABI  
3. **Use inline ABI**: Provide ABI as escaped JSON string for better reliability in MCP context
4. **Validate addresses**: Ensure all addresses/IDs are exactly 64 hex characters (32 bytes)
5. **Quote all strings**: Even if they look like numbers, string types need quotes
6. **Mind the nesting**: Complex nested structures require careful attention to bracket/brace matching
