# Forc

Forc stands for Fuel Orchestrator. Forc provides a variety of tools and commands for developers working with the Fuel ecosystem, such as scaffolding a new project, formatting, running scripts, deploying contracts, testing contracts, and more. If you're coming from a Rust background, `forc` is similar to `cargo`.

## Init (`forc init`)

```plaintext
$ forc init --help
forc-init 0.1.0
Create a new Forc project

USAGE:
    forc init <project-name>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

ARGS:
    <project-name>
```

Creates a new project from scratch, setting up all necessary files for a complete Fuel project.

```plaintext
$ forc init my-fuel-project
$ cd my-fuel-project
$ tree
.
├── Forc.toml
├── src
│   └── main.sw
└── tests
    ├── Cargo.toml
    └── harness.rs
```

`Forc.toml` is the Forc manifest file, containing information about the project and dependencies.

A `src/` directory is created, with a single `main.sw` Sway file in it.

A `tests/` directory is also created, however, this is a Rust package, that's why under it you can see a `Cargo.toml`, which is a Rust project manifest file. This manifest contains necessary Rust dependencies to enable you to write Rust-based tests using our Rust SDK (`fuels-rs`). More on this in the `Test` section down below.

## Build (`forc build`)

```plaintext
$ forc build --help
forc-build 0.1.0
Compile the current or target project

USAGE:
    forc build [FLAGS] [OPTIONS]

FLAGS:
    -h, --help                      Prints help information
        --offline                   Offline mode, prevents Forc from using the network when managing dependencies.
                                    Meaning it will only try to use previously downloaded dependencies
        --print-finalized-asm       Whether to compile to bytecode (false) or to print out the generated ASM (true)
        --print-intermediate-asm    Whether to compile to bytecode (false) or to print out the generated ASM (true)
    -s, --silent                    Silent mode. Don't output any warnings or errors to the command line
    -V, --version                   Prints version information

OPTIONS:
    -o <binary-outfile>        If set, outputs a binary file representing the script bytes
    -p, --path <path>          Path to the project, if not specified, current working directory will be used
```

Compiles Sway files.

```plaintext
$ forc build
  Compiled script "my-fuel-project".
  Bytecode size is 28 bytes.
```

## Test (`forc test`)

You can write tests in Rust using our [Rust SDK](https://github.com/FuelLabs/fuels-rs/). These tests can be run using either the Rust compiler / Cargo, or you can opt to use `forc test`, which will look for Rust tests under the `tests/` directory (which is created automatically with `forc init`).

For example, let's write tests against this contract, written in Sway:

```Rust
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

```Rust
use fuel_tx::Salt;
use fuels_abigen_macro::abigen;
use fuels_rs::contract::Contract;
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
    let compiled = Contract::compile_sway_contract("../", salt).unwrap();

    // Launch a local network and deploy the contract
    let (client, contract_id) = Contract::launch_and_deploy(&compiled).await.unwrap();

    let contract_instance = MyContract::new(compiled, client);

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

```plaintext
$ forc test

running 1 test
test harness ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.64s
```

Alternatively, you could `cd` into `test/` and run `cargo test`.

Alternatively, you could opt to write these tests in Typescript, using our [Typescript SDK](https://github.com/FuelLabs/fuels-ts/).

## Run (`forc run`)

```plaintext
$ forc run --help
forc-run 0.1.0
Run script project. Crafts a script transaction then sends it to a running node

USAGE:
    forc run [FLAGS] [OPTIONS] [node-url]

FLAGS:
        --dry-run                   Only craft transaction and print it out
    -h, --help                      Prints help information
    -k, --kill-node                 Kill Fuel Node Client after running the code. This is only available if the node is
                                    started from `forc run`
    -r, --pretty-print              Pretty-print the outputs from the node
        --print-finalized-asm       Whether to compile to bytecode (false) or to print out the generated ASM (true)
        --print-intermediate-asm    Whether to compile to bytecode (false) or to print out the generated ASM (true)
    -s, --silent                    Silent mode. Don't output any warnings or errors to the command line
    -V, --version                   Prints version information

OPTIONS:
    -o <binary-outfile>        If set, outputs a binary file representing the script bytes
    -d, --data <data>          Hex string of data to input to script
    -p, --path <path>          Path to the project, if not specified, current working directory will be used

ARGS:
    <node-url>    URL of the Fuel Client Node [env: FUEL_NODE_URL=]  [default: 127.0.0.1:4000]
```

## Deploy (`forc deploy`)

```plaintext
$ forc deploy --help
forc-deploy 0.1.0
Deploy contract project. Crafts a contract deployment transaction then sends it to a running node

USAGE:
    forc deploy [FLAGS] [OPTIONS]

FLAGS:
    -h, --help                      Prints help information
        --offline                   Offline mode, prevents Forc from using the network when managing dependencies.
                                    Meaning it will only try to use previously downloaded dependencies
        --print-finalized-asm       Whether to compile to bytecode (false) or to print out the generated ASM (true)
        --print-intermediate-asm    Whether to compile to bytecode (false) or to print out the generated ASM (true)
    -s, --silent                    Silent mode. Don't output any warnings or errors to the command line
    -V, --version                   Prints version information

OPTIONS:
    -o <binary-outfile>        If set, outputs a binary file representing the script bytes
    -p, --path <path>          Path to the project, if not specified, current working directory will be used
```

Alternatively, you could deploy your contract programmatically using our SDK:

```rust
// Build the contract
let salt: [u8; 32] = rng.gen();
let salt = Salt::from(salt);
let compiled = Contract::compile_sway_contract("../", salt).unwrap();

// Launch a local network and deploy the contract
let (client, contract_id) = Contract::launch_and_deploy(&compiled).await.unwrap();
```

## Update (`forc update`)

```plaintext
$ forc update --help
forc-update 0.1.0
Update dependencies in the Forc dependencies directory

USAGE:
    forc update [FLAGS] [OPTIONS]

FLAGS:
    -c, --check      Checks if the dependencies have newer versions. Won't actually perform the update, will output
                     which ones are up-to-date and outdated
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -p, --path <path>             Path to the project, if not specified, current working directory will be used
    -d <target-dependency>        Dependency to be updated. If not set, all dependencies will be updated
```

## Format (`forc fmt`)

```plaintext
$ forc fmt --help
forc-fmt 0.1.0
Format all Sway files of the current project

USAGE:
    forc fmt [FLAGS]

FLAGS:
    -c, --check      Run in 'check' mode. Exits with 0 if input is formatted correctly. Exits with 1 and prints a diff
                     if formatting is required
    -h, --help       Prints help information
    -V, --version    Prints version information
```

## Parse bytecode (`forc parse-bytecode`)

```plaintext
$ forc parse-bytecode --help
forc-parse-bytecode 0.1.0
Parse bytecode file into a debug format

USAGE:
    forc parse-bytecode <file-path>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

ARGS:
    <file-path>
```

Example with the initial project created using `forc init`:

```plaintext
$ forc build -o obj
  Compiled script "my-fuel-project".
  Bytecode size is 28 bytes.
```

```plaintext
my-second-project forc parse-bytecode obj

 half-word  byte  op               raw                notes
         0  0     JI(4)            [144, 0, 0, 4]     conditionally jumps to byte 16
         1  4     NOOP             [71, 0, 0, 0]
         2  8     Undefined        [0, 0, 0, 0]       data section offset lo (0)
         3  12    Undefined        [0, 0, 0, 28]      data section offset hi (28)
         4  16    LW(46, 12, 1)    [93, 184, 192, 1]
         5  20    ADD(46, 46, 12)  [16, 186, 227, 0]
         6  24    RET(0)           [36, 0, 0, 0]
```
