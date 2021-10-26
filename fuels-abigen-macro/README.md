# Fuels-rs Abigen macro

Fuels-rs' Abigen is a procedural macro used to transform a contract's ABI defined as a JSON object into type-safe Rust bindings, i.e. Rust structs and types that represent that contract's ABI. These bindings are then expanded and brought into scope.

The specifications for the JSON ABI format and its encoding/decoding can be found [here](https://github.com/FuelLabs/fuel-specs/blob/master/specs/protocol/abi.md#json-abi-format).

## Usage

A simple example of generating type-safe bindings from a JSON ABI specified in-line:

```rust
fn compile_bindings_from_inline_contract() {
    // Generates the bindings from the an ABI definition inline.
    // The generated bindings can be accessed through `SimpleContract`.
    abigen!(
        SimpleContract,
        r#"
        [
            {
                "type": "contract",
                "inputs": [
                    {
                        "name": "arg",
                        "type": "u32"
                    },
                    {
                        "name": "second_arg",
                        "type": "u16"
                    }
                ],
                "name": "takes_ints_returns_bool",
                "outputs": [
                    {
                        "name": "",
                        "type": "bool"
                    }
                ]
            }
        ]
        "#
    );

    let contract_instance = SimpleContract::new();

    let contract_call = contract_instance.takes_ints_returns_bool(42 as u32, 10 as u16);

    let encoded = format!(
        "{}{}",
        contract_call.encoded_selector, contract_call.encoded_params
    );

    assert_eq!("0000000003b568d4000000000000002a000000000000000a", encoded);
}
```

This example and many more can be found under `tests/harness.rs`. To run the whole test suite run `cargo test` inside `fuels-abi-gen-macro/`.
