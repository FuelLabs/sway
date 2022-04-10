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

Compile the sway files of the current project.

```console
$ forc build
Compiled script "my-fuel-project".
Bytecode size is 28 bytes.
```

The output produced will depend on the project's program type. Building script, predicate and contract projects will produce their bytecode in binary format `<project-name>.bin`. Building contracts and libraries will also produce the public ABI in JSON format `<project-name>-abi.json`.

By default, these artifacts are placed in the `out/` directory.

If a `Forc.lock` file did not yet exist, it will be created in order to pin each of the dependencies listed in `Forc.toml` to a specific commit or version.

## Update (`forc update`)

Updates each of the dependencies so that they point to the latest suitable commit or version given their dependency declaration. The result is written to the `Forc.lock` file.

## Test (`forc test`)

You can write tests in Rust using our [Rust SDK](https://github.com/FuelLabs/fuels-rs). These tests can be run using `forc test`, which will look for Rust tests under the `tests/` directory (which is created automatically with `forc init`).

For example, let's write tests against this contract, written in Sway:

```rust
contract;

abi TestContract {
    fn initialize_counter(value: u64) -> u64;
    fn increment_counter(amount: u64) -> u64;
}

storage {
    counter: u64
}

impl TestContract for Contract {
    fn initialize_counter(value: u64) -> u64 {
        storage.counter = value;
        value
    }

    fn increment_counter(amount: u64) -> u64 {
        let incremented = storage.counter + amount;
        storage.counter = incremented;
        incremented
    }
}
```

Our `tests/harness.rs` file could look like:

```rust
use fuel_tx::Salt;
use fuels_abigen_macro::abigen;
use fuels_contract::{contract::Contract, parameters::TxParameters};
use fuels_signers::util::test_helpers;

abigen!(
    MyContract,
    "out/debug/my_contract-abi.json"
);

#[tokio::test]
async fn harness() {
    let salt = Salt::from([0u8; 32]);
    let compiled =
        Contract::load_sway_contract("out/debug/my_contract.bin", salt)
            .unwrap();

    let (provider, wallet) = test_helpers::setup_test_provider_and_wallet().await;
    let id = Contract::deploy(&compiled, &provider, &wallet, TxParameters::default())
        .await
        .unwrap();

    let instance = MyContract::new(id.to_string(), provider, wallet);

    // Call `initialize_counter()` method in our deployed contract.
    // Note that, here, you get type-safety for free!
    let result = instance.initialize_counter(42)
            .call()
            .await
            .unwrap();
    assert_eq!(42, result.value);

    // Call `increment_counter()` method in our deployed contract.
    let result = instance.increment_counter(10)
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

Alternatively, you could opt to write these tests in Typescript, using our [Typescript SDK](https://github.com/FuelLabs/fuels-ts).

## Run (`forc run`)

Run script project. Crafts a script transaction then sends it to a running node.

## Deploy (`forc deploy`)

Deploy contract project. Crafts a contract deployment transaction then sends it to a running node.

Alternatively, you could deploy your contract programmatically using our SDK:

```rust
// Load the contract
let salt = Salt::from([0u8; 32]);
let compiled = Contract::load_sway_contract("out/debug/my_contract.bin", salt).unwrap();

// Launch a local network and deploy the contract
let (provider, wallet) = test_helpers::setup_test_provider_and_wallet().await;
let id = Contract::deploy(&compiled, &provider, &wallet, TxParameters::default())
    .await
    .unwrap();
```

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
