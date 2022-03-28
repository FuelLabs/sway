# Test

## SYNOPSIS

`forc test` [_test_name_]

## DESCRIPTION

Run Rust-based tests on current project. As of now, `forc test` is a simple wrapper on `cargo test`.

You can write tests in Rust using our Rust SDK. These tests can be run using forc test, which will look for Rust tests under the tests/ directory (which is created automatically with `forc init`).

For example, let's write tests against this contract, written in Sway:

```rust
contract;

use std::storage::store_u64;
use std::storage::get_u64;

abi TestContract {
  fn initialize_counter(gas_: u64, amount_: u64, coin_: b256, value: u64) -> u64;
  fn increment_counter(gas_: u64, amount_: u64, coin_: b256, amount: u64) -> u64;
}

const COUNTER_KEY = 0x0000000000000000000000000000000000000000000000000000000000000000;

impl TestContract for Contract {
  fn initialize_counter(gas_: u64, amount_: u64, color_: b256, value: u64) -> u64 {
    store_u64(COUNTER_KEY, value);
    value
  }
  fn increment_counter(gas_: u64, amount_: u64, color_: b256, amount: u64) -> u64 {
    let value = get_u64(COUNTER_KEY) + amount;
    store_u64(COUNTER_KEY, value);
    value
  }
}
```

Our `tests/harness.rs` file could look like:

```rust
use fuel_tx::Salt;
use fuels_abigen_macro::abigen;
use fuels_contract::contract::Contract;
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};

// Generate Rust bindings from our contract JSON ABI
abigen!(MyContract, "./my-contract-abi.json");

#[tokio::test]
async fn harness() {
    let rng = &mut StdRng::seed_from_u64(2322u64);

    // Build the contract
    let salt: [u8; 32] = rng.gen();
    let salt = Salt::from(salt);

    let compiled = Contract::compile_sway_contract("./", salt).unwrap();
    let client = Provider::launch(Config::local_node()).await.unwrap();
    let contract_id = Contract::deploy(&compiled, &client).await.unwrap();
    println!("Contract deployed @ {:x}", contract_id);

    let contract_instance = MyContract::new(contract_id.to_string(), client);

    // Call `initialize_counter()` method in our deployed contract.
    // Note that, here, you get type-safety for free!
    let result = contract_instance
        .initialize_counter(42)
        .call()
        .await
        .unwrap();

    assert_eq!(42, result.unwrap());

    // Call `increment_counter()` method in our deployed contract.
    let result = contract_instance
        .increment_counter(10)
        .call()
        .await
        .unwrap();

    assert_eq!(52, result.unwrap());
}
```

Then, in the root of our project, running `forc test` will run the test above, compiling and deploying the contract to a local Fuel network, and calling the ABI methods against the contract deployed in there:

```console
$ forc test

running 1 test
test harness ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.64s
```

Alternatively, you could opt to write these tests in Typescript, using our [Typescript SDK](https://github.com/FuelLabs/fuels-ts).
