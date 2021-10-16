use fuels_abigen::abigen;

// TODO: Find some asserts to put here

#[test]
fn compile_bindings_from_contract_file() {
    // Generates the bindings from an ABI definition in a JSON file
    // The generated bindings can be accessed through `SimpleContract`.
    abigen!(
        SimpleContract,
        "fuels-abigen/tests/takes_ints_returns_bool.json"
    );

    // `SimpleContract` is the name of the contract
    let contract_instance = SimpleContract::new();

    // Calls the function defined in the JSON ABI.
    // Note that this is type-safe, if the function does exist
    // in the JSON ABI, this won't compile!
    // Currently this prints `0000000003b568d4000000000000002a000000000000000a`
    // The encoded contract call. Soon it'll be able to perform the
    // actual call.
    let _function = contract_instance.takes_ints_returns_bool(42 as u32, 10 as u16);

    // Then you'll be able to use `.call()` to actually call the contract with the
    // specified function:
    // function.call().unwrap();
    // Or you might want to just `contract_instance.takes_u32_returns_bool(42 as u32).call()?`
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

    let _function = contract_instance.takes_ints_returns_bool(42 as u32, 10 as u16);
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
                "type": "contract",
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

    let contract_instance = SimpleContract::new();

    let _function = contract_instance.takes_ints_returns_bool(42 as u32);
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

    let contract_instance = SimpleContract::new();

    let input: Vec<u16> = vec![1, 2, 3, 4];
    let _function = contract_instance.takes_array(input);
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

    let contract_instance = SimpleContract::new();

    let input: Vec<bool> = vec![true, false, true];
    let _function = contract_instance.takes_array(input);
}

// TODO: continue from here. Test Byte, B256, String, then the ones I know are failing
// for sure: Struct and Enum
