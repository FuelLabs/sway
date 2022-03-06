# Deploy and Call a Sway Contract With Rust

This guide walks through the steps for deploying and calling a Sway contract with the Rust SDK. You can go [here](https://github.com/FuelLabs/fuels-rs) for full documentation on the Rust SDK.

## Deploy `wallet_contract` with the Rust SDK

Now, in your Rust application, copy and paste the following code to set up a local node, compile and deploy the `wallet_contract`.

```rust
// Setup a local node
let node = FuelService::new_node(Config::local_node()).await.unwrap();
let client = FuelClient::from(node.bound_address);

// Compile the contract
let salt: [u8; 32] = rng.gen();
let salt = Salt::from(salt);

let compiled =
    Contract::compile_sway_contract("path/to/your/fuel/wallet_contract", salt).unwrap();

// Deploy the contract
let contract_id = Contract::deploy(compiled_contract, client).await.unwrap();
```

## Run `wallet_script` Against You Local Fuel Node

Now, in your Rust application, copy and paste the following code to actually run the script:

```rust
let compiled = Script::compile_sway_script("path/to/fuel/wallet_script").unwrap();

let tx = Transaction::Script {
    gas_price: 0,
    gas_limit: 1_000_000,
    maturity: 0,
    receipts_root: Default::default(),
    script: compiled.raw, // Here we pass the compiled script into the transaction
    script_data: vec![],
    inputs: vec![],
    outputs: vec![],
    witnesses: vec![vec![].into()],
    metadata: None,
};

let script = Script::new(tx);

let result = script.call(&client).await.unwrap();
```
