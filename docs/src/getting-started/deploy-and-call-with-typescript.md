# Deploy and call a Sway contract with Typescript

This guide walks through the steps for deploying and calling a Sway contract in Typescript. Go [here]() for full documentation on the Typescript SDK.

## 7. Deploy `wallet_contract` with Typescript SDK

In your Typescript application, copy and paste the following code to set up a local node, compile and deploy the `wallet_contract`.

```rust
    // Setup a local node
    let server = FuelService::new_node(Config::local_node()).await.unwrap();
    let client = FuelClient::from(srv.bound_address);

    // Compile the contract
    let salt: [u8; 32] = rng.gen();
    let salt = Salt::from(salt);

    let compiled =
        Contract::compile_sway_contract("path/to/your/fuel/wallet_contract", salt).unwrap();

    // Deploy the contract
    let contract_id = Contract::deploy(compiled_contract, fuel_client).await.unwrap();
```

## 8. Write a Sway script to call a Sway smart contract

Now that we have deployed our wallet contract, we need to actually _call_ our contract. We can do this by calling the contract from a script.

Let's navigate to the `wallet_script` repo created in step 2.

First, you need to link the `wallet_lib` library. Open up the `Forc.toml` in the root of the repo. It should look something like this:

```toml
[project]
authors = ["Yiren Lu"]
entry = "main.sw"
license = "Apache-2.0"
name = "wallet_script"

[dependencies]
core = { git = "http://github.com/FuelLabs/sway-lib-core" }
std = { git = "http://github.com/FuelLabs/sway-lib-std" }
```

Link the `wallet_lib` repo by adding the following line to the bottom of the file:

```toml
wallet_lib = {path = "../wallet_lib"}
```

Next, open up the `main.sw` file in `src`. Copy and paste the following code:

```sway
script;

use wallet_abi::Wallet;
use wallet_abi::SendFundsRequest;
use std::constants::ETH_ID;

fn main() {
    let caller = abi(Wallet, contract_address);
    let req = SendFundsRequest {
        amount_to_send: 200,
        recipient_address: 0x9299da6c73e6dc03eeabcce242bb347de3f5f56cd1c70926d76526d7ed199b8b,
    };
    caller.send_funds(10000, 0, ETH_ID, req);
}
```

Replace the `$contract_address` with the contract id you noted in step 8.

The main new concept is the _abi cast_: `abi(AbiName, contract_address)`. This returns a `ContractCaller` type which can be used to call contracts. The methods of the ABI become the methods available on this contract caller: `send_funds` and `receive_funds`. We then construct the request format, `SendFundsRequest`, and directly call the contract ABI method as if it was just a regular method.

## 9. Check that `wallet_script` builds.

To check that `wallet_script` builds successfully, run

```console
forc build
```

from the root of the `wallet_script` directory.

## 10. Run `wallet_script`

Now, in your Rust application, copy and paste the following code to set up a local node, compile and deploy the wallet contract.

```rust
     let compiled =
    Script::compile_sway_script("path/to/fuel/wallet_script")
        .unwrap();

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

```

```
