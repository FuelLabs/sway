# Forc

Forc stands for Fuel Orchestrator. Forc provides a variety of tools and commands for developers working with the Fuel ecosystem, such as scaffolding a new project, formatting, running scripts, deploying contracts, testing contracts, and more. If you're coming from a Rust background, `forc` is similar to `cargo`.

## Init (`forc init`)

Creates a new project from scratch, setting up all necessary files for a complete Fuel project.

```console
$ forc init my-fuel-project
$ cd my-fuel-project
$ tree
.
├── Cargo.toml
├── Forc.toml
├── src
│   └── main.sw
└── tests
    └── harness.rs
```

`Forc.toml` is the Forc manifest file, containing information about the project and dependencies. `Cargo.toml` is the Rust project manifest file, used by the Rust-based tests package.

A `src/` directory is created, with a single `main.sw` Sway file in it.

A `tests/` directory is also created. The `Cargo.toml` in the root directory contains necessary Rust dependencies to enable you to write Rust-based tests using our Rust SDK (`fuels-rs`). More on this in the `Test` section down below.

## Build (`forc build`)

Compiles Sway files.

```console
$ forc build
Compiled script "my-fuel-project".
Bytecode size is 28 bytes.
```

## Test (`forc test`)

You can write tests in Rust using our [Rust SDK](https://github.com/FuelLabs/fuels-rs). These tests can be run using `forc test`, which will look for Rust tests under the `tests/` directory (which is created automatically with `forc init`).

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

## Run (`forc run`)

Run script project. Crafts a script transaction then sends it to a running node.

## Deploy (`forc deploy`)

Deploy contract project. Crafts a contract deployment transaction then sends it to a running node.

Alternatively, you could deploy your contract programmatically using our SDK:

```rust
// Build the contract
let salt: [u8; 32] = rng.gen();
let salt = Salt::from(salt);
let compiled = Contract::compile_sway_contract("./", salt).unwrap();

// Launch a local network and deploy the contract
let compiled = Contract::compile_sway_contract("./", salt).unwrap();
let client = Provider::launch(Config::local_node()).await.unwrap();
let contract_id = Contract::deploy(&compiled, &client).await.unwrap();
```

## Update (`forc update`)

Update dependencies in the Forc dependencies directory.

## Format (`forc fmt`)

Format all Sway files of the current project.

## Parse bytecode (`forc parse-bytecode`)

Parse bytecode file into a debug format.

Example with the initial project created using `forc init`:

```console
$ forc build -o obj
Compiled script "my-fuel-project".
Bytecode size is 28 bytes.
```

```console
my-second-project$ forc parse-bytecode obj

 half-word  byte  op               raw                notes
         0  0     JI(4)            [144, 0, 0, 4]     conditionally jumps to byte 16
         1  4     NOOP             [71, 0, 0, 0]
         2  8     Undefined        [0, 0, 0, 0]       data section offset lo (0)
         3  12    Undefined        [0, 0, 0, 28]      data section offset hi (28)
         4  16    LW(46, 12, 1)    [93, 184, 192, 1]
         5  20    ADD(46, 46, 12)  [16, 186, 227, 0]
         6  24    RET(0)           [36, 0, 0, 0]
```
