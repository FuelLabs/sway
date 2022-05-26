# Testing with Rust

If you look again at the project structure when you create a [new Forc project](../introduction/forc_project.md) with `forc new`, you can see a directory called `tests/`:

```plaintext
$ forc new my-fuel-project
$ cd my-fuel-project
$ tree .
├── Cargo.toml
├── Forc.toml
├── src
│   └── main.sw
└── tests
    └── harness.rs
```

Note that this is also a Rust package, hence the existence of a `Cargo.toml` (Rust manifest file) in the project root directory. The `Cargo.toml` in the root directory contains necessary Rust dependencies to enable you to write Rust-based tests using our [Rust SDK](https://github.com/FuelLabs/fuels-rs), (`fuels-rs`).

These tests can be run using `forc test` which will look for Rust tests under the `tests/` directory (created automatically with `forc new`).

For example, let's write tests against the following contract, written in Sway. This can be done in the pregenerated `src/main.sw` or in a new file in `src`. In the case of the latter, update the `entry` field in `Forc.toml` to point at the new contract.

```sway
{{#include ../../../examples/counter/src/main.sw}}
```

Our `tests/harness.rs` file could look like:

<!--TODO add test here once examples are tested-->

```rust,ignore
use fuel_tx::{ContractId, Salt};
use fuels::prelude::*;
use fuels::test_helpers;
use fuels_abigen_macro::abigen;

// Load abi from json
abigen!(MyContract, "out/debug/my-fuel-project-abi.json");

async fn get_contract_instance() -> (MyContract, ContractId) {
    // Deploy the compiled contract
    let salt = Salt::from([0u8; 32]);
    let compiled = Contract::load_sway_contract("./out/debug/my-fuel-project.bin", salt).unwrap();

    // Launch a local network and deploy the contract
    let (provider, wallet) = test_helpers::setup_test_provider_and_wallet().await;

    let id = Contract::deploy(&compiled, &provider, &wallet, TxParameters::default())
        .await
        .unwrap();

    let instance = MyContract::new(id.to_string(), provider, wallet);

    (instance, id)
}

#[tokio::test]
async fn can_get_contract_id() {
    let (contract_instance, _id) = get_contract_instance().await;
    // Now you have an instance of your contract you can use to test each function

    // Call `initialize_counter()` method in our deployed contract.
    // Note that, here, you get type-safety for free!
    let result = contract_instance
        .initialize_counter(42)
        .call()
        .await
        .unwrap();

    assert_eq!(42, result.value);

    // Call `increment_counter()` method in our deployed contract.
    let result = contract_instance
        .increment_counter(10)
        .call()
        .await
        .unwrap();

    assert_eq!(52, result.value);
}
```

Then, in the root of our project, running `forc test` will run the test above, compiling and deploying the contract to a local Fuel network, and calling the ABI methods against the contract deployed in there:

```console
$ forc test

running 1 test
test harness ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.64s
```
