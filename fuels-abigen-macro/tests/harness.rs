use fuels_abigen_macro::abigen;
use fuels_rs::contract::Contract;
use sha2::{Digest, Sha256};

const MOCK_CONTRACT_ADDRESS: &'static str =
    "e50103684750e4916cd9825b14cf7e6763ffcc6523a9e0af63de93dbd6e3d736";

#[test]
fn compile_bindings_from_contract_file() {
    // Generates the bindings from an ABI definition in a JSON file
    // The generated bindings can be accessed through `SimpleContract`.
    abigen!(
        SimpleContract,
        "fuels-abigen-macro/tests/takes_ints_returns_bool.json"
    );

    // `SimpleContract` is the name of the contract
    let contract_instance = SimpleContract::new(MOCK_CONTRACT_ADDRESS.into());

    // Calls the function defined in the JSON ABI.
    // Note that this is type-safe, if the function does exist
    // in the JSON ABI, this won't compile!
    // Currently this prints `0000000003b568d4000000000000002a000000000000000a`
    // The encoded contract call. Soon it'll be able to perform the
    // actual call.
    let contract_call = contract_instance.takes_ints_returns_bool(42 as u32, 10 as u16);

    // Then you'll be able to use `.call()` to actually call the contract with the
    // specified function:
    // function.call().unwrap();
    // Or you might want to just `contract_instance.takes_u32_returns_bool(42 as u32).call()?`

    let encoded = format!(
        "{}{}",
        contract_call.encoded_selector, contract_call.encoded_params
    );

    assert_eq!("0000000003b568d4000000000000002a000000000000000a", encoded);
}

#[test]
fn compile_bindings_from_inline_contract() {
    // Generates the bindings from the an ABI definition inline.
    // The generated bindings can be accessed through `SimpleContract`.
    abigen!(
        SimpleContract,
        r#"
        [
            {
                "type": "function",
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

    let contract_instance = SimpleContract::new(MOCK_CONTRACT_ADDRESS.into());

    let contract_call = contract_instance.takes_ints_returns_bool(42 as u32, 10 as u16);

    let encoded = format!(
        "{}{}",
        contract_call.encoded_selector, contract_call.encoded_params
    );

    assert_eq!("0000000003b568d4000000000000002a000000000000000a", encoded);
}

#[test]
fn compile_bindings_single_param() {
    // Generates the bindings from the an ABI definition inline.
    // The generated bindings can be accessed through `SimpleContract`.
    abigen!(
        SimpleContract,
        r#"
        [
            {
                "type": "function",
                "inputs": [
                    {
                        "name": "arg",
                        "type": "u32"
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

    let contract_instance = SimpleContract::new(MOCK_CONTRACT_ADDRESS.into());

    let contract_call = contract_instance.takes_ints_returns_bool(42 as u32);

    let encoded = format!(
        "{}{}",
        contract_call.encoded_selector, contract_call.encoded_params
    );

    assert_eq!("000000009593586c000000000000002a", encoded);
}

#[test]
fn compile_bindings_array_input() {
    // Generates the bindings from the an ABI definition inline.
    // The generated bindings can be accessed through `SimpleContract`.
    abigen!(
        SimpleContract,
        r#"
        [
            {
                "type":"contract",
                "inputs":[
                    {
                        "name":"arg",
                        "type":"u16[3]"
                    }
                ],
                "name":"takes_array",
                "outputs":[

                ]
            }
        ]
        "#
    );

    let contract_instance = SimpleContract::new(MOCK_CONTRACT_ADDRESS.into());

    let input: Vec<u16> = vec![1, 2, 3, 4];
    let contract_call = contract_instance.takes_array(input);

    let encoded = format!(
        "{}{}",
        contract_call.encoded_selector, contract_call.encoded_params
    );

    assert_eq!(
        "00000000f0b878640000000000000001000000000000000200000000000000030000000000000004",
        encoded
    );
}

#[test]
fn compile_bindings_bool_array_input() {
    // Generates the bindings from the an ABI definition inline.
    // The generated bindings can be accessed through `SimpleContract`.
    abigen!(
        SimpleContract,
        r#"
        [
            {
                "type":"contract",
                "inputs":[
                    {
                        "name":"arg",
                        "type":"bool[3]"
                    }
                ],
                "name":"takes_array",
                "outputs":[

                ]
            }
        ]
        "#
    );

    let contract_instance = SimpleContract::new(MOCK_CONTRACT_ADDRESS.into());

    let input: Vec<bool> = vec![true, false, true];
    let contract_call = contract_instance.takes_array(input);

    let encoded = format!(
        "{}{}",
        contract_call.encoded_selector, contract_call.encoded_params
    );

    assert_eq!(
        "00000000f8fe942c000000000000000100000000000000000000000000000001",
        encoded
    );
}

#[test]
fn compile_bindings_byte_input() {
    // Generates the bindings from the an ABI definition inline.
    // The generated bindings can be accessed through `SimpleContract`.
    abigen!(
        SimpleContract,
        r#"
        [
            {
                "type":"contract",
                "inputs":[
                    {
                        "name":"arg",
                        "type":"byte"
                    }
                ],
                "name":"takes_byte",
                "outputs":[

                ]
            }
        ]
        "#
    );

    let contract_instance = SimpleContract::new(MOCK_CONTRACT_ADDRESS.into());

    let contract_call = contract_instance.takes_byte(10 as u8);

    let encoded = format!(
        "{}{}",
        contract_call.encoded_selector, contract_call.encoded_params
    );

    assert_eq!("00000000a4bd3861000000000000000a", encoded);
}

#[test]
fn compile_bindings_string_input() {
    // Generates the bindings from the an ABI definition inline.
    // The generated bindings can be accessed through `SimpleContract`.
    abigen!(
        SimpleContract,
        r#"
        [
            {
                "type":"contract",
                "inputs":[
                    {
                        "name":"arg",
                        "type":"str[23]"
                    }
                ],
                "name":"takes_string",
                "outputs":[

                ]
            }
        ]
        "#
    );

    let contract_instance = SimpleContract::new(MOCK_CONTRACT_ADDRESS.into());

    let contract_call = contract_instance.takes_string("This is a full sentence".into());

    let encoded = format!(
        "{}{}",
        contract_call.encoded_selector, contract_call.encoded_params
    );

    assert_eq!(
        "00000000d56e76515468697320697320612066756c6c2073656e74656e636500",
        encoded
    );
}

#[test]
fn compile_bindings_b256_input() {
    // Generates the bindings from the an ABI definition inline.
    // The generated bindings can be accessed through `SimpleContract`.
    abigen!(
        SimpleContract,
        r#"
        [
            {
                "type":"contract",
                "inputs":[
                    {
                        "name":"arg",
                        "type":"b256"
                    }
                ],
                "name":"takes_b256",
                "outputs":[

                ]
            }
        ]
        "#
    );

    let contract_instance = SimpleContract::new(MOCK_CONTRACT_ADDRESS.into());

    let mut hasher = Sha256::new();
    hasher.update("test string".as_bytes());

    let arg = hasher.finalize();

    let contract_call = contract_instance.takes_b256(arg.into());

    let encoded = format!(
        "{}{}",
        contract_call.encoded_selector, contract_call.encoded_params
    );

    assert_eq!(
        "0000000054992852d5579c46dfcc7f18207013e65b44e4cb4e2c2298f4ac457ba8f82743f31e930b",
        encoded
    );
}

#[test]
fn compile_bindings_struct_input() {
    // Generates the bindings from the an ABI definition inline.
    // The generated bindings can be accessed through `SimpleContract`.
    abigen!(
        SimpleContract,
        r#"
        [
            {
                "type":"contract",
                "inputs":[
                    {
                        "name":"MyStruct",
                        "type":"struct",
                        "components": [
                            {
                                "name": "foo",
                                "type": "u8"
                            },
                            {
                                "name": "bar",
                                "type": "bool"
                            }
                        ]
                    }
                ],
                "name":"takes_struct",
                "outputs":[]
            }
        ]
        "#
    );

    // Because of the abigen! macro, `MyStruct` is now in scope
    // and can be used!
    let input = MyStruct {
        foo: 10 as u8,
        bar: true,
    };

    let contract_instance = SimpleContract::new(MOCK_CONTRACT_ADDRESS.into());

    let contract_call = contract_instance.takes_struct(input);

    let encoded = format!(
        "{}{}",
        contract_call.encoded_selector, contract_call.encoded_params
    );

    assert_eq!("00000000f5957fce000000000000000a0000000000000001", encoded);
}

#[test]
fn compile_bindings_nested_struct_input() {
    // Generates the bindings from the an ABI definition inline.
    // The generated bindings can be accessed through `SimpleContract`.
    abigen!(
        SimpleContract,
        r#"
        [
            {
                "type":"contract",
                "inputs":[
                    {
                        "name":"MyNestedStruct",
                        "type":"struct",
                        "components": [
                            {
                                "name": "x",
                                "type": "u16"
                            },
                            {
                                "name": "inner_struct",
                                "type": "struct",
                                "components": [
                                    {
                                        "name":"a",
                                        "type": "bool"
                                    }
                                ]
                            }
                        ]
                    }
                ],
                "name":"takes_nested_struct",
                "outputs":[]
            }
        ]
        "#
    );

    let inner_struct = InnerStruct { a: true };

    let input = MyNestedStruct {
        x: 10 as u16,
        inner_struct,
    };

    let contract_instance = SimpleContract::new(MOCK_CONTRACT_ADDRESS.into());

    let contract_call = contract_instance.takes_nested_struct(input);

    let encoded = format!(
        "{}{}",
        contract_call.encoded_selector, contract_call.encoded_params
    );

    assert_eq!("00000000e8a04d9c000000000000000a0000000000000001", encoded);
}

#[test]
fn compile_bindings_enum_input() {
    // Generates the bindings from the an ABI definition inline.
    // The generated bindings can be accessed through `SimpleContract`.
    abigen!(
        SimpleContract,
        r#"
        [
            {
                "type":"contract",
                "inputs":[
                    {
                        "name":"MyEnum",
                        "type":"enum",
                        "components": [
                            {
                                "name": "x",
                                "type": "u32"
                            },
                            {
                                "name": "y",
                                "type": "bool"
                            }
                        ]
                    }
                ],
                "name":"takes_enum",
                "outputs":[]
            }
        ]
        "#
    );

    let variant = MyEnum::X(42);

    let contract_instance = SimpleContract::new(MOCK_CONTRACT_ADDRESS.into());

    let contract_call = contract_instance.takes_enum(variant);

    let encoded = format!(
        "{}{}",
        contract_call.encoded_selector, contract_call.encoded_params
    );

    assert_eq!("000000009542a3c90000000000000000000000000000002a", encoded);
}

#[tokio::test]
async fn example_workflow() {
    // This test case

    // Generates the bindings from the an ABI definition inline.
    // The generated bindings can be accessed through `MyContract`.
    abigen!(
        MyContract,
        r#"
        [
            {
                "type": "function",
                "inputs": [
                    {
                        "name": "arg",
                        "type": "u32"
                    }
                ],
                "name": "takes_int",
                "outputs": [
                ]
            }
        ] 
        "#
    );

    // Build the contract
    let compiled = Contract::compile_sway_contract("tests/test_projects/contract_test").unwrap();

    // Launch local network and deploy contract to testnet.
    // Note that if `false` was passed to `stop_node`,
    // `launch_and_deploy` would return a child process
    // and we would be responsible for killing this process once
    // we're done with testing.
    // This is useful in case of long-lived local tests, spanning
    // across different contracts being deployed and interacted with in
    // the same session.
    let (_, contract_id, logs) = Contract::launch_and_deploy(compiled, true).await.unwrap();

    println!("Contract deployed @ 0x{}", contract_id);
    println!("logs: {:?}\n", logs);

    let contract_instance = MyContract::new(contract_id);

    let _contract_call = contract_instance.takes_int(42 as u32);

    // TODO: Send the actual transaction
    // contract_call.send().await?;
}
