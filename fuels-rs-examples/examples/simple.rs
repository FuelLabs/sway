use fuels_abigen::abigen;

fn main() {
    let contract = r#"
        [
            {
                "type":"contract",
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
        "#;

    abigen!(SimpleContract, contract);

    // `test` is the name of the contract
    let contract_instance = test::new();

    // Calls the function defined in the JSON ABI.
    // Note that this is type-safe, if the function does exist
    // in the JSON ABI, this won't compile!
    // Currently this prints `000000006355e6ee000000000000002a`
    // The encoded contract call. Soon it'll be able to perform the
    // actual call.
    let _function = contract_instance.takes_u32_returns_bool(42 as u32);

    // Then you'll be able to use `.call()` to actually call the contract with the
    // specified function:
    // function.call().unwrap();
    // Or you might want to just `contract_instance.takes_u32_returns_bool(42 as u32).call()?`
}
