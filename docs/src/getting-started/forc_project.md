# A Forc Project

To initialize a new project with Forc, use `forc init`:

```console
forc init hello_world
```

Here is the project that Forc has initialized:

```console
$ cd hello_world
$ tree .
├── Cargo.toml
├── Forc.toml
├── src
│   └── main.sw
└── tests
    └── harness.rs
```

`Forc.toml` is the _manifest file_ (similar to `Cargo.toml` for Cargo or `package.json` for Node), and defines project metadata such as the project name and dependencies.

```toml
[project]
name = "hello_world"
author = "user"
entry = "main.sw"
license = "Apache-2.0"

[dependencies]
core = { git = "http://github.com/FuelLabs/sway-lib-core" }
std = { git = "http://github.com/FuelLabs/sway-lib-std" }
```

Here are the contents of the only Sway file in the project, and the main entry point, `src/main.sw`:

```sway
script;

fn main() {
    
}
```

The project is _script_, one of four different project types. For additional information on different project types, see [here](./../sway-on-chain/index.md).

We now compile our project with `forc build`, passing the flag `--print-finalized-asm` to view the generated assembly:

```console
$ forc build --print-finalized-asm
.program:
ji   i4
noop
DATA_SECTION_OFFSET[0..32]
DATA_SECTION_OFFSET[32..64]
lw   $ds $is 1
add  $ds $ds $is
ret  $zero                    ; main fn returns unit value
.data:

Compiled script "hello_world".
Bytecode size is 28 bytes.
```

To run this script, use `forc run` (note that `fuel-core` must be running for this to work):

```console
$ forc run
Bytecode size is 28 bytes.
[Return { id: ContractId([0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]), val: 0, pc: 488, is: 464 }]
```

Use `forc json-abi` to output the ABI of the contract. To write this to a `.json` file (which is necessary for running tests below), pipe it using something like `forc json-abi > my_contract.json`. There is currently not a convention for where ABI files should be placed; one common choice is loose in the root directory.

## Testing a Sway Project with Forc

If you look again at the project structure when you create a new Forc project, you can see a directory called `tests/`:

```plaintext
$ forc init my-fuel-project
$ cd my-fuel-project
$ tree .
├── Cargo.toml
├── Forc.toml
├── src
│   └── main.sw
└── tests
    └── harness.rs
```

Note that this is a Rust package, hence the existence of a `Cargo.toml` (Rust manifest file) in the project root directory. The `Cargo.toml` in the root directory contains necessary Rust dependencies to enable you to write Rust-based tests using our [Rust SDK](https://github.com/FuelLabs/fuels-rs) (`fuels-rs`).

These tests can be run using either `carg test`, or `forc test` which will look for Rust tests under the `tests/` directory (created automatically with `forc init`).

For example, let's write tests against the following contract, written in Sway. This can be done in the pregenerated `src/main.sw` or in a new file in `src`. In the case of the latter, update the `entry` field in `Forc.toml` to point at the new contract.

```sway
contract;

use std::storage::*;
use std::constants::*;

abi TestContract {
    fn initialize_counter(gas_: u64, amount_: u64, coin_: b256, value: u64) -> u64;
    fn increment_counter(gas_: u64, amount_: u64, coin_: b256, amount: u64) -> u64;
}

const SLOT = 0x0000000000000000000000000000000000000000000000000000000000000000;

impl TestContract for Contract {
    fn initialize_counter(gas_: u64, amount_: u64, color_: b256, value: u64) -> u64 {
        store(SLOT, value);
        value
    }

    fn increment_counter(gas_: u64, amount_: u64, color_: b256, amount: u64) -> u64 {
        let storedVal: u64 = get(SLOT);
        let value = storedVal + amount;
        store(SLOT, value);
        value
    }
}
```

Our `tests/harness.rs` file could look like:

```rust
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
    let compiled = Contract::compile_sway_contract("./", salt).unwrap();

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

    assert_eq!(42, result);

    // Call `increment_counter()` method in our deployed contract.
    let result = contract_instance
        .increment_counter(10)
        .call()
        .await
        .unwrap();

    assert_eq!(52, result);
}
```

Then, in the root of our project, running `forc test` or `cargo test` will run the test above, compiling and deploying the contract to a local Fuel network, and calling the ABI methods against the contract deployed in there:

```plaintext
$ forc test

running 1 test
test harness ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.64s
```

Instead of writing tests in Rust, tests can also be written in Typescript using our [Typescript SDK](https://github.com/FuelLabs/fuels-ts/).
