# A Forc Project

To initialize a new project with Forc, use `forc init`:

```console
forc init hello_world
```

Here is the project that Forc has initialized:

```console
$ cd hello_world
$ tree .
├── Forc.toml
├── Cargo.toml
├── src
│   └── main.sw
└── tests
    └── harness.rs
```

`Forc.toml` is the _manifest file_ (similar to `Cargo.toml` for Cargo or `package.json` for Node), and defines project metadata such as the project name and dependencies.

```toml
[project]
author  = "user"
license = "MIT"
name = "hello_world"
entry = "main.sw"
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

## Testing a Sway project with Forc

If you look again at the project structure when you create a new Forc project, you can see a directory called `tests/`:

```plaintext
$ forc init my-fuel-project
$ cd my-fuel-project
$ tree
.
├── Forc.toml
├── Cargo.toml
├── src
│   └── main.sw
└── tests
    └── harness.rs
```

Note that this is a Rust package, that's why inside it you can see a `Cargo.toml`, which is a Rust project manifest file. The `Cargo.toml` in the root directory contains necessary Rust dependencies to enable you to write Rust-based tests using our Rust SDK (`fuels-rs`).

These tests can be run using either the Rust compiler / Cargo, or you can opt to use `forc test`, which will look for Rust tests under the `tests/` directory (which is created automatically with `forc init`).

For example, let's write tests against this contract, written in Sway:

```sway
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

```sway
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
