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

Note that this is also a Rust package, hence the existence of a `Cargo.toml` (Rust manifest file) in the project root directory. The `Cargo.toml` in the root directory contains necessary Rust dependencies to enable you to write Rust-based tests using our [Rust SDK](https://fuellabs.github.io/fuels-rs/latest), (`fuels-rs`).

These tests can be run using `forc test` which will look for Rust tests under the `tests/` directory (created automatically with `forc new` and prepopulated with boilerplate code).

For example, let's write tests against the following contract, written in Sway. This can be done in the pregenerated `src/main.sw` or in a new file in `src`. In the case of the latter, update the `entry` field in `Forc.toml` to point at the new contract.

```sway
{{#include ../../../examples/counter/src/main.sw}}
```

Our `tests/harness.rs` file could look like:

<!--TODO add test here once examples are tested-->

```rust,ignore
use fuels::{prelude::*, tx::ContractId};

// Load abi from json
abigen!(TestContract, "out/debug/my-fuel-project-abi.json");

async fn get_contract_instance() -> (TestContract, ContractId) {
    // Launch a local network and deploy the contract
    let mut wallets = launch_custom_provider_and_get_wallets(
        WalletsConfig::new(
            Some(1),             /* Single wallet */
            Some(1),             /* Single coin (UTXO) */
            Some(1_000_000_000), /* Amount per coin */
        ),
        None,
    )
    .await;
    let wallet = wallets.pop().unwrap();

    let id = Contract::deploy(
        "./out/debug/my-fuel-project.bin",
        &wallet,
        TxParameters::default(),
        StorageConfiguration::with_storage_path(Some(
            "./out/debug/my-fuel-project-storage_slots.json".to_string(),
        )),
    )
    .await
    .unwrap();

    let instance = TestContractBuilder::new(id.to_string(), wallet).build();

    (instance, id.into())
}

#[tokio::test]
async fn can_get_contract_id() {
    let (contract_instance, _id) = get_contract_instance().await;
    // Now you have an instance of your contract you can use to test each function

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
...
running 1 test
test can_get_contract_id ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.22s
```
