# Getting Started

Follow this guide to write and deploy a simple wallet smart contract in Sway.

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

Every Sway file must begin with a declaration of what type of program it is.

See [the chapter on program types](../sway-program-types/index.md) for more information.

## Create Wallet Projects with `forc`

To deploy a wallet on Fuel, we will need to write a library, a contract, and a script in Sway.

First, let's [install the Fuel toolchain](./installation.md). Then with `forc` installed, let's create three different sibling projects:

```sh
forc new wallet_lib
forc new wallet_contract
forc new wallet_script
```

See [here](./forc_project.md) for more information on Forc project structure.

## Write a Sway Smart Contract

### Declare ABI in `wallet_lib`

Navigate into the `src/main.sw` file of the `wallet_lib` directory you just created.

Delete the auto-generated skeleton code currently in the file, and copy and paste the following code:

```sway
library wallet_lib;

abi Wallet {
    fn receive_funds();
    fn send_funds(amount_to_send: u64, recipient_address: b256);
}
```

Every Sway file must start with a declaration of what type of program the file contains; here, we've declared that this file is a `library` called `wallet_lib`.

Sway contracts should declare an ABI—an **a**pplication **b**inary **i**nterface—in a library so that it can be re-used by downstream contracts. Let's focus on the ABI declaration and inspect it line-by-line.

In the first line, we declare the name of this ABI: `Wallet`. To import this ABI into either a script or another contract for calling the contract, or the contract to implement the ABI, you would use `use wallet_lib::Wallet;`.

In the second line we declare an ABI method called `receive_funds` which, when called, should receive funds into this wallet. This method takes no parameters and does not return anything.

> **Note**: We are simply defining an interface here, so there is no function body or implementation of the function. We only need to define the interface itself. In this way, ABI declarations are similar to [trait declarations](../advanced/traits.md).

In the third line we declare another ABI method, this time called `send_funds`. It takes two parameters: the amount to send, and the address to send the funds to.

### Implementing the ABI Methods in `wallet_contract`

Now that we've defined the interface, let's discuss how to use it. We will start by implementing the above ABI for a specific contract.

To do this, navigate to the `wallet_contract` directory that you created with `forc` previously.

First, you need to import the `Wallet` declaration from the last step. Open up `Forc.toml`. It should look something like this:

```toml
[project]
authors = ["user"]
entry = "main.sw"
license = "Apache-2.0"
name = "wallet_contract"

[dependencies]
```

Include the `wallet_lib` project as a dependency by adding the following line to the bottom of the file:

```toml
wallet_lib = { path = "../wallet_lib" }
```

Now, open up `main.sw` in `wallet_contract/src` and copy and paste the following code:

```sway
contract;
use wallet_lib::Wallet;

impl Wallet for Contract {
    fn receive_funds() {
    }

    fn send_funds(amount_to_send: u64, recipient_address: b256) {
    }
}
```

This implements the ABI methods with empty bodies. Actual implementation of the bodies is left as an exercise for the reader.

## Build the Contract

Build `wallet_contract` by running

```sh
forc build
```

from inside the `wallet_contract` directory.

## Deploy the Contract

It's now time to deploy the wallet contract and call it on a Fuel node. We will show how to do this using `forc` from the command line, but you can also do it using the [Rust SDK](https://github.com/FuelLabs/fuels-rs#deploying-a-sway-contract) or the [TypeScript SDK](https://github.com/FuelLabs/fuels-ts/#deploying-contracts)

### Spin Up a Fuel node

In a separate tab in your terminal, spin up a local Fuel node:

```sh
fuel-core --db-type in-memory
```

This starts a Fuel node with a volatile database that will be cleared when shut down (good for testing purposes).

### Deploy `wallet_contract` To Your Local Fuel Node

To deploy `wallet_contract` on your local Fuel node, run

```sh
forc deploy
```

from the root of the `wallet_contract` directory.

This should produce some output in `stdout` that looks like this:

```console
$ forc deploy
  Compiled library "wallet_lib".
  Compiled contract "wallet_contract".
  Bytecode size is 212 bytes.
Contract id: 0xf4b63e0e09cb72762cec18a6123a9fb5bd501b87141fac5835d80f5162505c38
Logs:
HexString256(HexFormatted(0xd9240bc439834bc6afc3f334abf285b3b733560b63d7ce1eb53afa8981984af7))
```

Note the contract ID—you will need it in the next step.

## Write a Sway Script to Call a Sway Contract

> **Note**: If you are using the SDK you do not need to write a script to call the Sway contract, this is all handled automagically by the SDK.

Now that we have deployed our wallet contract, we need to actually _call_ our contract. We can do this by calling the contract from a script.

Let's navigate to the `wallet_script` directory created previously.

First, you need to import the `wallet_lib` library. Open up the `Forc.toml` in the root of the directory. Import `wallet_lib` repo by adding the following line to the bottom of the file:

```toml
wallet_lib = { path = "../wallet_lib" }
```

Next, open up `src/main.sw`. Copy and paste the following code:

```sway
script;

use std::constants::BASE_ASSET_ID;

use wallet_lib::Wallet;

fn main() {
    let caller = abi(Wallet, <contract_address>);
    caller.send_funds(200, 0x9299da6c73e6dc03eeabcce242bb347de3f5f56cd1c70926d76526d7ed199b8b);
}
```

Replace `<contract_address>` with the contract ID you noted when deploying the contract.

The main new concept is the _abi cast_: `abi(AbiName, ContractAddress)`. This returns a `ContractCaller` type which can be used to call contracts. The methods of the ABI become the methods available on this contract caller: `send_funds` and `receive_funds`. We can directly call a contract ABI method as if it were a trait method.

## Check That `wallet_script` Builds

To check that `wallet_script` builds successfully, run

```sh
forc build
```

from the root of the `wallet_script` directory.

## Call the Contract

It's now time to call the contract. We will show how to do this using `forc` from the command line, but you can also do this using the [Rust SDK](https://github.com/FuelLabs/fuels-rs#generating-type-safe-rust-bindings) or the [TypeScript SDK](https://github.com/FuelLabs/fuels-ts/#calling-contracts)

### Run `wallet_script` Against Your Local Fuel Node

To run the script now against the local Fuel node, run

```sh
forc run --contract <contract-id>
```

from the root of the `wallet_script` directory.

Note that we are passing in the `wallet_contract` contract ID as a command-line parameter. You will need to pass in the contract ID of every contract that this script will be interacting with.

If the script is successfully run, it will output something that looks like:

```console
$ forc run --pretty-print --contract <contract-id>
  Compiled library "core".
  Compiled library "std".
  Compiled library "wallet_lib".
  Compiled script "wallet_script".
  Bytecode size is 272 bytes.
[
  {
    "Call": {
      "amount": 0,
      "asset_id": "0000000000000000000000000000000000000000000000000000000000000000",
      "gas": 99999240,
      "id": "ea1f774aae16b8719ce463d4e8097ef72766686ede65e35947084aa0055e59d7",
      "is": 11536,
      "param1": 3467577331,
      "param2": 10848,
      "pc": 11536,
      "to": "ea1f774aae16b8719ce463d4e8097ef72766686ede65e35947084aa0055e59d7"
    }
  },
  {
    "Return": {
      "id": "ea1f774aae16b8719ce463d4e8097ef72766686ede65e35947084aa0055e59d7",
      "is": 11536,
      "pc": 11608,
      "val": 0
    }
  },
  {
    "Return": {
      "id": "0000000000000000000000000000000000000000000000000000000000000000",
      "is": 10352,
      "pc": 10476,
      "val": 0
    }
  },
  {
    "ScriptResult": {
      "gas_used": 971,
      "result": "Success"
    }
  }
]
```

It returns a `Call` receipt and a `ScriptResult` receipt.

## Testing Sway contracts

The recommended way to test Sway contracts is via the [Rust SDK](../testing/testing-with-rust.md). You may also write tests in TypeScript if you are using the TypeScript SDK.
