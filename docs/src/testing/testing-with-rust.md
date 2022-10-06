# Testing with Rust

A common use of Sway is for writing contracts or scripts that exist as part of a
wider Rust application. In order to test the interaction between our Sway code
and our Rust code we can add integration testing.

## Adding Rust Integration Testing

To add Rust integration testing to a Forc project we can use [the `sway-test-rs`
cargo generate
template](https://github.com/FuelLabs/sway/tree/master/templates/sway-test-rs).
This template makes it easy for Sway devs to add the boilerplate required when
setting up their Rust integration testing.

Let's add a Rust integration test to [the fresh project we created in the
introduction](../introduction/forc_project.md).

### 1. Enter the project

To recap, here's what our empty project looks like:

```console
$ cd my-fuel-project
$ tree .
├── Forc.toml
└── src
    └── main.sw
```

### 2. Install `cargo generate`

We're going to add a Rust integration test harness using a cargo generate
template. Let's make sure we have the `cargo generate` command installed!

```console
$ cargo install cargo-generate
```

> _**Note**: You can learn more about cargo generate by visiting [its
> repository](https://github.com/cargo-generate/cargo-generate)._

### 3. Generate the test harness

Let's generate the default test harness with the following:

```
cargo generate --init fuellabs/sway templates/sway-test-rs --name my-fuel-project
```

If all goes well, the output should look as follows:

```console
⚠️   Favorite `fuellabs/sway` not found in config, using it as a git repository: https://github.com/fuellabs/sway
🤷   Project Name : my-fuel-project
🔧   Destination: /home/user/path/to/my-fuel-project ...
🔧   Generating template ...
[1/3]   Done: Cargo.toml
[2/3]   Done: tests/harness.rs
[3/3]   Done: tests
🔧   Moving generated files into: `/home/user/path/to/my-fuel-project`...
✨   Done! New project created /home/user/path/to/my-fuel-project
```

Let's have a look at the result:

```console
$ tree .
├── Cargo.toml
├── Forc.toml
├── src
│   └── main.sw
└── tests
    └── harness.rs
```

We have two new files!

- The `Cargo.toml` is the manifest for our new test harness and specifies the
  required dependencies including `fuels` the Fuel Rust SDK.
- The `tests/harness.rs` contains some boilerplate test code to get us started,
  though doesn't call any contract methods just yet.

### 4. Build the forc project

Before running the tests, we need to build our contract so that the necessary
ABI, storage and bytecode artifacts are available. We can do so with `forc
build`:

```console
$ forc build
  Creating a new `Forc.lock` file. (Cause: lock file did not exist)
    Adding core
    Adding std git+https://github.com/fuellabs/sway?tag=v0.24.5#e695606d8884a18664f6231681333a784e623bc9
   Created new lock file at /home/user/path/to/my-fuel-project/Forc.lock
  Compiled library "core".
  Compiled library "std".
  Compiled contract "my-fuel-project".
  Bytecode size is 60 bytes.
```

At this point, our project should look like the following:

```console
$ tree
├── Cargo.toml
├── Forc.lock
├── Forc.toml
├── out
│   └── debug
│       ├── my-fuel-project-abi.json
│       ├── my-fuel-project.bin
│       └── my-fuel-project-storage_slots.json
├── src
│   └── main.sw
└── tests
    └── harness.rs
```

We now have an `out` directory with our required JSON files!

> _**Note**: This step may no longer be required in the future as we plan to
> enable the integration testing to automatically build the artifacts as
> necessary so that files like the ABI JSON are always up to date._

### 5. Build and run the tests

Now we're ready to build and run the default integration test.

```console
$ cargo test
    Updating crates.io index
   Compiling version_check v0.9.4
   Compiling proc-macro2 v1.0.46
   Compiling quote v1.0.21
   ...
   Compiling fuels v0.24.0
   Compiling my-fuel-project v0.1.0 (/home/user/path/to/my-fuel-project)
    Finished test [unoptimized + debuginfo] target(s) in 1m 03s
     Running tests/harness.rs (target/debug/deps/integration_tests-373971ac377845f7)

running 1 test
test can_get_contract_id ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.36s
```

> _**Note**: The first time we run `cargo test`, cargo will spend some time
> fetching and building the dependencies for Fuel's Rust SDK. This might take a
> while, but only the first time!_

If all went well, we should see some output that looks like the above!


## Writing Tests

Now that we've learned how to setup Rust integration testing in our project,
let's try to write some of our own tests!

First, let's update our contract code with a simple counter example:

```sway
{{#include ../../../examples/counter/src/main.sw}}
```

To test our `initialize_counter` and `increment_counter` contract methods from
the Rust test harness, we could update our `tests/harness.rs` file with the
following:

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
async fn initialize_and_increment() {
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

Let's build our project once more and run the test:

```console
$ forc build
```
```console
$ cargo test
   Compiling my-fuel-project v0.1.0 (/home/mindtree/programming/sway/my-fuel-project)
    Finished test [unoptimized + debuginfo] target(s) in 11.61s
     Running tests/harness.rs (target/debug/deps/integration_tests-373971ac377845f7)

running 1 test
test initialize_and_increment ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.25s
```

When cargo runs our test, our test uses the SDK to spin up a local in-memory
Fuel network, deploy our contract to it, and call the contract methods via the
ABI.

You can add as many functions decorated with `#[tokio::test]` as you like, and
`cargo test` will automatically test each of them!
