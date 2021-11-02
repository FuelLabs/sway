# Sway ABI CLI

Simple CLI program to encode Sway function calls and decode their output. The ABI being encoded and decoded is specified [here](https://github.com/FuelLabs/fuel-specs/blob/master/specs/protocol/abi.md).

## Usage

```plaintext
sway-abi-cli 0.1.0
Sway/Fuel ABI coder

USAGE:
    sway-abi-cli <SUBCOMMAND>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

SUBCOMMANDS:
    decode    Decode ABI call result
    encode    Encode ABI call
    help      Prints this message or the help of the given subcommand(s)
```

## Examples

You can choose to encode only the given params or you can go a step further and have a full JSON ABI file and encode the whole input to a certain function call defined in the JSON file.

### Encoding params only

```plaintext
cargo run -- encode params -v bool true
```

```plaintext
0000000000000001
```

```plaintext
cargo run -- encode params -v bool true -v u32 42 -v u32 100
```

```plaintext
0000000000000001000000000000002a0000000000000064
```

Note that for every param you want to encode, you must pass a `-v` flag followed by the type, and then the value: `-v <type_1> <value_1> -v <type_2> <value_2> -v <type_n> <value_n>`

### Encoding function call

`example/simple.json`:

```json
[
  {
    "type":"function",
    "inputs":[
      {
        "name":"arg",
        "type":"u32"
      }
    ],
    "name":"takes_u32_returns_bool",
    "outputs":[
      {
        "name":"",
        "type":"bool"
      }
    ]
  }
]
```

```plaintext
cargo run -- encode function examples/simple.json takes_u32_returns_bool -p 4
```

```plaintext
000000006355e6ee0000000000000004
```

`example/array.json`

```json
[
  {
    "type":"function",
    "inputs":[
      {
        "name":"arg",
        "type":"u16[3]"
      }
    ],
    "name":"takes_array",
    "outputs":[
      {
        "name":"",
        "type":"u16[2]"
      }
    ]
  }
]
```

```plaintext
cargo run -- encode function examples/array.json takes_array -p '[1,2]'
```

```plaintext
00000000f0b8786400000000000000010000000000000002
```

Note that the first word (8 bytes) of the output is reserved for the function selector, which is captured in the last 4 bytes, which is simply the 256hash of the function signature.

Example with nested struct:

```json
[
  {
    "type":"contract",
    "inputs":[
      {
        "name":"MyNestedStruct",
        "type":"struct",
        "components":[
          {
            "name":"x",
            "type":"u16"
          },
          {
            "name":"y",
            "type":"struct",
            "components":[
              {
                "name":"a",
                "type":"bool"
              },
              {
                "name":"b",
                "type":"u8[2]"
              }
            ]
          }
        ]
      }
    ],
    "name":"takes_nested_struct",
    "outputs":[
      
    ]
  }
]
```

```plaintext
cargo run -- encode function examples/nested_struct.json takes_nested_struct -p '(10, (true, [1,2]))'
```

```
00000000e8a04d9c000000000000000a000000000000000100000000000000010000000000000002
```

### Decoding params only

Similar to encoding parameters only:

```plaintext
cargo run -- decode params -t bool -t u32 -t u32 0000000000000001000000000000002a0000000000000064
```

```plaintext
Bool(true)
U32(42)
U32(100)
```

### Decoding function output 

```plaintext
cargo run -- decode function examples/simple.json takes_u32_returns_bool 0000000000000001
```

```plaintext
Bool(true)
```
