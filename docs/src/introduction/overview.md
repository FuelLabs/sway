# Getting Started

Follow this guide to write, test, and deploy a simple smart contract in Sway.

## Glossary

Before we begin, it may be helpful to understand terminology that will used throughout the docs and how they relate to each other:

- **Fuel**: the Fuel blockchain.
- **FuelVM**: the virtual machine powering Fuel.
- **Sway**: the domain-specific language crafted for the FuelVM; it is inspired by Rust.
- **Forc**: the build system and package manager for Sway, similar to Cargo for Rust.

## Understand Sway Program Types

There are four types of Sway programs:

- `contract`
- `predicate`
- `script`
- `library`

Contracts, predicates, and scripts can produce artifacts usable on the blockchain, while a library is simply a project designed for code reuse and is not directly deployable.

See [the chapter on program types](../sway-program-types/index.md) for more information.

## Your First Sway Project

We'll build a simple counter contract with two functions: one to increment the counter, and one to return the value of the counter.

A few pieces of info that will be helpful before moving on:

- The main features of a smart contract that differentiate it from scripts or predicates are that it is callable and stateful.
- A script is runnable bytecode on the chain which can call contracts to perform some task. It does not represent ownership of any resources and it cannot be called by a contract.

### Writing the Contract

First, let's [install the Sway toolchain](./installation.md). Then with `forc` installed, create a contract project:

```sh
forc new counter_contract
```

Here is the project that Forc has initialized:

```console
$ cd counter_contract
$ tree .
├── Cargo.toml
├── Forc.toml
├── src
│   └── main.sw
└── tests
    └── harness.rs
```

`Forc.toml` is the _manifest file_ (similar to `Cargo.toml` for Cargo or `package.json` for Node), and defines project metadata such as the project name and dependencies.

We'll be writing our code in the `src/main.sw`.

`cd` (change directories) into your contract project and delete the boilerplate code in `src/main.sw`. Every Sway file must start with a declaration of what type of program the file contains; here, we've declared that this file is a contract.

```sway
contract;
```

Next, we'll define a our storage value. In our case, we have a single counter that we'll call `counter` of type 64-bit unsigned integer and initialize it to 0.

```sway
storage {
    counter: u64 = 0,
}
```

### ABI

An ABI defines an interface, and there is no function body in the ABI. A contract must either define or import an ABI declaration and implement it. It is considered best practice to define your ABI in a separate library and import it into your contract because this allows callers of the contract to import and use the ABI in scripts to call your contract.

For simplicity, we will define the ABI natively in the contract.

```sway
abi Counter {
    #[storage(read, write)]
    fn increment();

    #[storage(read)]
    fn counter() -> u64;
}
```

### Going line by line

`#[storage(read, write)]` is an annotation which denotes that this function has the permissions to read and write a value in storage.

`fn increment()` - We're introducing the functionality to increment and denoting it shouldn't return any value.

`#[storage(read)]` is an annotation which denotes that this function has the permissions to read values in storage.

`fn counter() -> u64;` - We're introducing the functionality to to increment the counter and denoting the function's return value.

### Implement ABI

Below your ABI definition, you will write the implementation of the functions defined in your ABI.

```sway
impl Counter for Contract {
    #[storage(read)]
    fn counter() -> u64 {
        return storage.counter;
      }
    #[storage(read, write)]
    fn increment(){
        storage.counter = storage.counter + 1;
    }
}
```

> Note: `return storage.counter;` is equivalent to `storage.counter`.

### What we just did

Read and return the counter property value from the contract storage.

```sway
fn counter() -> u64 {
    return storage.counter;
}
```

The function body accesses the value counter in storage, and increments the value by one. Then, we return the newly updated value of counter.

```sway
fn increment() {
        storage.counter = storage.counter + 1;
}
```

### Build the Contract

Build `counter_contract` by running the following command in your terminal from inside the `counter_contract` directory:

```sh
forc build
```

You should see something like this output:

```console
Compiled library "core".
  Compiled library "std".
  Compiled contract "counter_contract".
  Bytecode size is 240 bytes.
```

### Deploy the Contract

It's now time to deploy the contract and call it on a Fuel node. We will show how to do this using `forc` from the command line, but you can also do it using the [Rust SDK](https://github.com/FuelLabs/fuels-rs#deploying-a-sway-contract) or the [TypeScript SDK](https://github.com/FuelLabs/fuels-ts/#deploying-contracts)

### Spin Up a Fuel node

In a separate tab in your terminal, spin up a local Fuel node:

```sh
fuel-core --db-type in-memory
```

This starts a Fuel node with a volatile database that will be cleared when shut down (good for testing purposes).

### Deploy `counter_contract` To Your Local Fuel Node

To deploy `counter_contract` on your local Fuel node, open a new terminal tab and run the following command from the root of the `wallet_contract` directory:

```sh
forc deploy
```

> Note you can't use the same terminal session that is running fuel-core to run any other commands as this will end your fuel-core process.

This should produce some output in `stdout` that looks like this:

```console
$ forc deploy
  Compiled library "core".
  Compiled library "std".
  Compiled contract "counter_contract".
  Bytecode size is 208 bytes.
Contract id: 0x1d64105ed60f22f3def36ebbda45d58513e69bcbc4b2fcce0875898b0468d276
Logs:
TransactionId(HexFormatted(69a1c45f31892f61ae6b67edd8524550769b1432b7f1984ca0a456ea0de18da7))
```

Note the contract ID—you will need it if you want to build out a frontend to interact with this contract.

## Testing your Contract

In the directory `tests`, navigate to `harness.rs.` Here you'll see there is some boilerplate code to help you start interacting with and testing your contract.

At the bottom of the file, define the body of `can_get_contract_instance`. Here is what your code should look like to verify that the value of the counter did get incremented:

```sway
#[tokio::test]
async fn can_get_contract_id() {
    let (_instance, _id) = get_contract_instance().await;
    // Now you have an instance of your contract you can use to test each function
    let result = _instance.increment().call().await.unwrap();
    assert!(result.value > 0)
}
```

Run the following command in the terminal: `forc test`.

You'll see something like this as your output:

```console
 Compiled library "core".
  Compiled library "std".
  Compiled contract "counter_contract".
  Bytecode size is 208 bytes.
  Compiling counter_contract v0.1.0 (/Users/camiinthisthang/Desktop/Workspace/Fuel/counter_contract)
  Finished test [unoptimized + debuginfo] target(s) in 11.71s
  Running tests/harness.rs (target/debug/deps/integration_tests-6a600a6a87f48edb)

running 1 test
test can_get_contract_id ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.24s
```

Congratulations, you've just created and tested your first Sway smart contract 🎉. Now you can build a frontend to interact with your contract using the TypeScript SDK. You can find a step-by-step guide to building a front end for your project [here](https://fuellabs.github.io/fuels-ts/QUICKSTART.html).
