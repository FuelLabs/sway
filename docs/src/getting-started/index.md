# Getting Started

Follow this guide to write and deploy a simple wallet smart contract in Sway.

## 0. Glossary

Before we begin, it may be helpful to understand terminology that will used throughout the docs and how they relate to each other:

- **Fuel** - refers to the Fuel blockchain and virtual machine
- **Sway** - refers to domain-specific language crafted for the Fuel virtual machine; it is inspired by Rust
- **Forc** - refers to the build system and package manager for Sway, similar to Cargo for Rust.

Go [here](./sway-toolchain.md) for more detailed information about each of these components

## 1. Installing Fuel and Sway

### a) Install Rust as a dependency

A prerequisite for installing and using Sway is the Rust toolchain. Platform-specific instructions can be found [here](https://www.rust-lang.org/tools/install).

### b) Install other system dependencies

#### MacOS

```console
brew update
brew install openssl cmake llvm libpq postgresql
```

#### Debian

```console
apt update
apt install -y cmake pkg-config libssl-dev git gcc build-essential git clang libclang-dev llvm libpq-dev
```

#### Arch

```console
pacman -Syu --needed --noconfirm cmake gcc openssl-1.0 pkgconf git clang llvm11 llvm11-libs postgresql-libs
export OPENSSL_LIB_DIR="/usr/lib/openssl-1.0";
export OPENSSL_INCLUDE_DIR="/usr/include/openssl-1.0"
```

### c) Install the Sway toolchain and Fuel full node with Cargo

```console
cargo install forc fuel-core
```

Go [here](./installation) for instructions on how to install fuel-core from source.

## 2. Understand Sway program types

There are four types of Sway programs:

- contract
- predicate
- script
- library

Contracts, predicates, and scripts are all deployable on the blockchain. A library is simply a project designed for code reuse and is never directly deployed to the chain.

Every Sway file must begin with a declaration of what type of program it is.

## 3. Create wallet projects with forc

To deploy a wallet on the fuel blockchain, we will need to write a library, a contract, and a script in Sway.

To start, let's create three different sibling projects with `forc`:

```console
forc init wallet_lib
forc init wallet_contract
forc init wallet_script
```

If you step into any of these directories, this is the hierarchy that has been auto-generated.

```console
$ cd wallet_lib
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
{{#include ../../../examples/hello_world/Forc.toml}}
```

Here are the contents of the only Sway file in the project, and the main entry point, `src/main.sw`:

```sway
script;

fn main() {

}
```

## 3. Write a Sway smart contract

### a) Declare ABI in `wallet_lib`

Navigate into the `main.sw` file of the `wallet_lib` directory you just created.

```console
cd wallet_lib/src
```

Delete the auto-generated code currently in the file and copy and paste the following code:

```sway
    library wallet_lib;

    abi Wallet {
        fn receive_funds(gas: u64, coins_to_forward: u64, asset_id: b256, unused: ());
        fn send_funds(gas: u64, coins_to_forward: u64, asset_id: b256, req: SendFundsRequest);
    }

    pub struct SendFundsRequest {
        amount_to_send: u64,
        recipient_address: b256,
    }
```

Every Sway file starts with a declaration of what type of program the file contains; here, we've declared that this file contains a library.

Next, every Sway smart contract must declare an ABI - an application binary interface interface. If we think of a smart contract as a deployed API with some state, then the abi is like a Thrift file that defines the services that the API implements.

If we look at the code above, there are two declarations. One is a struct representing the data that `send_funds` needs and the other is the ABI declaration. Let's focus on the ABI declaration and inspect it line-by-line:

```sway
abi Wallet {
    fn receive_funds(gas: u64, coins_to_forward: u64, asset_id: b256, unused: ());
    fn send_funds(gas: u64, coins_to_forward: u64, asset_id: b256, req: SendFundsRequest);
}
```

In the first line, `abi Wallet {`, we declare the name of this _Application Binary Interface_, or ABI. We are naming this ABI `Wallet`. To import this ABI into either a script for calling or a contract for implementing, you would use `use wallet_abi::Wallet;`.

In the second line,

```sway
    fn receive_funds(gas: u64, coins_to_forward: u64, asset_id: b256, unused: ());
```

we are declaring an ABI interface surface method called `receive_funds` which, when called, should receive funds into this wallet. This method takes four parameters:

1. `gas` represents the gas being forwarded to the contract when it is called.
2. `coins_to_forward` represents how many coins are being forwarded with this call.
3. `asset_id` represents the ID of the _asset type_ of the coin being forwarded.
4. `unused` is the configurable user parameter, which this method does not need and is therefore unused.

and does not return anything.

**For now, all ABI methods must take these four parameters _in this order_. This will change shortly, and ABI methods will be able to accept any number of user-based parameters and not need to specify arguments for gas and coin forwarding. You will see a compile error if you do not specify these parameters correctly in your ABI.**

Note that we are simply defining an interface here, so there is no _function body_ or implementation of the function. We only need to define the interface itself. In this way, ABI declarations are similar to [trait declarations](../advanced/traits.md).

In the third line,

```sway
    fn send_funds(gas: u64, coins_to_forward: u64, asset_id: b256, req: SendFundsRequest);
```

we are declaring another ABI method, this time called `send_funds`. It takes the same parameters as the last ABI method, but with one difference: the fourth argument, the configurable one, is used. By specifying a struct here, you can pass in many values in this one parameter. In this case, `SendFundsRequest` simply has two values: the amount to send, and the address to send the funds to.

### b) Implementing the ABI methods in `wallet_contract`

Now that we'ved defined the interface, let's discuss how to use it. We will start by implementing the above ABI for a specific contract.

To do this, navigate to the `wallet_contract` repo that you created with `forc` in step 2.

```console
cd wallet_contract
```

First, you need to link the `Wallet` declaration from the last step. Open up the `Forc.toml`. It should look something like this:

```toml
[project]
authors = ["Yiren Lu"]
entry = "main.sw"
license = "Apache-2.0"
name = "wallet_contract"

[dependencies]
core = { git = "http://github.com/FuelLabs/sway-lib-core" }
std = { git = "http://github.com/FuelLabs/sway-lib-std" }
```

Link the `wallet_lib` repo by adding the following line to the bottom of the file:

```toml
wallet_lib = {path = "../wallet_lib"}
```

Now, open up the `main.sw` file in `wallet_contract/src` and copy and paste the following code:

```sway
impl Wallet for Contract {
    fn receive_funds(gas_to_forward: u64, coins_to_forward: u64, asset_id: b256, unused: ()) {
        if asset_id == ETH_ID {
            let balance = storage.balance.write();
            deref balance = balance + coins_to_forward;
        };
    }

    fn send_funds(gas_to_forward: u64, coins_to_forward: u64, asset_id: b256, req: SendFundsRequest) {
        assert(sender() == OWNER_ADDRESS);
        assert(storage.balance.read() > req.amount_to_send);
        let balance = storage.balance.write();
        deref balance = balance - req.amount_to_send;
        transfer_coins(asset_id, req.recipient_address, req.amount_to_send);
    }
}
```

This implements the ABI methods.

## 4. Build the Sway smart contract

Build `wallet_contract` by running

```console
forc build
```

from inside the `wallet_contract` repo.

## 5. Deploy the Sway contract

It's now time to deploy the wallet contract and call it on a Fuel node. You have a couple options of how to do the next few steps:

- [Using the Rust SDK](./deploy-and-call-with-rust.md)
- [Using the Typescript SDK](./deploy-and-call-with-typescript.md)
- [Using fuel-core from the command line](./deploy-and-call-with-cli.md)

If you are building an application on the Fuel blockchain, you will likely want to choose the Rust or Typescript SDK.
