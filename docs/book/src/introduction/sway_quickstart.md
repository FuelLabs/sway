# Sway Quickstart

Follow this guide to write and deploy a simple smart contract in Sway.

Check out the [Developer Quickstart Guide](https://fuellabs.github.io/fuel-docs/master/developer-quickstart.html) for a step-by-step guide on building a fullstack dapp on Fuel. The guide will walk you through writing a smart contract, setting up a wallet, and building a frontend to interact with your contract.

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
├── Forc.toml
└── src
    └── main.sw
```

`Forc.toml` is the _manifest file_ (similar to `Cargo.toml` for Cargo or `package.json` for Node), and defines project metadata such as the project name and dependencies.

We'll be writing our code in the `src/main.sw`.

`cd` (change directories) into your contract project and delete the boilerplate code in `src/main.sw`. Every Sway file must start with a declaration of what type of program the file contains; here, we've declared that this file is a contract.

```sway
contract;
```

Next, we'll define a storage value. In our case, we have a single counter that we'll call `counter` of type 64-bit unsigned integer and initialize it to 0.

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

`#[storage(read, write)]` is an annotation which denotes that this function has permission to read and write a value in storage.

`fn increment()` - We're introducing the functionality to increment and denoting it shouldn't return any value.

`#[storage(read)]` is an annotation which denotes that this function has permission to read values in storage.

`fn counter() -> u64;` - We're introducing the functionality to increment the counter and denoting the function's return value.

### Implement ABI

Below your ABI definition, you will write the implementation of the functions defined in your ABI.

```sway
impl Counter for Contract {
    #[storage(read)]
    fn counter() -> u64 {
      return storage.counter;
    }
    #[storage(read, write)]
    fn increment() {
        storage.counter = storage.counter + 1;
    }
}
```

> **Note**
> `return storage.counter;` is equivalent to `storage.counter`.

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
  Bytecode size is 224 bytes.
```

### Deploy the Contract

It's now time to deploy the contract and call it on a Fuel node. We will show how to do this using `forc` from the command line, but you can also do it using the [Rust SDK](https://fuellabs.github.io/fuels-rs/master/getting-started/contracts.html) or the [TypeScript SDK](https://fuellabs.github.io/fuels-ts/#deploying-contracts)

### Spin Up a Fuel node

In a separate tab in your terminal, spin up a local Fuel node:

```sh
fuel-core run --db-type in-memory
```

This starts a Fuel node with a volatile database that will be cleared when shut down (good for testing purposes).

### Deploy `counter_contract` To Your Local Fuel Node

> **Note**
> If you want to deploy your contract to the testnet instead of to a local Fuel node, check out the [Developer Quickstart Guide](https://fuellabs.github.io/fuel-docs/master/developer-quickstart.html).

To deploy `counter_contract` on your local Fuel node, open a new terminal tab and run the following command from the root of the `wallet_contract` directory:

```sh
forc deploy --unsigned
```

> **Note**
> You can't use the same terminal session that is running fuel-core to run any other commands as this will end your fuel-core process.

This should produce some output in `stdout` that looks like this:

```console
$ forc deploy --unsigned
  Compiled library "core".
  Compiled library "std".
  Compiled contract "counter_contract".
  Bytecode size is 224 bytes.
Contract id: 0xaf94c0a707756caae667ee43ca18bace441b25998c668010192444a19674dc4f
Logs:
TransactionId(HexFormatted(7cef24ea33513733ab78c5daa5328d622d4b38187d0f0d1857b272090d99f96a))
```

Note the contract ID — you will need it if you want to build out a frontend to interact with this contract.

## Testing Your Contract

We will cover how to test your contract later but, if you are eager to take a look, see [Unit Testing](../testing/unit-testing.md) and [Testing with Rust](../testing/testing-with-rust.md).

## Next Steps

Now that you've written a smart contract with Sway and deployed it to a local Fuel node, try out building a fullstack dapp deployed to the testnet. A step-by-step guide to write your smart contract, deploy to testnet, set up a wallet, and build a frontend can be found in the [Developer Quickstart Guide](https://fuellabs.github.io/fuel-docs/master/developer-quickstart.html).
