# Scripts

A script is an executable that does not need to be deployed because it only exists during a transaction.

It can be used to replicate the functionality of contracts, such as routers, without the cost of deployment or increase of the blockchain size.

Some properties of a script include:

- It cannot be called by a contract
- It is stateless but can interact with storage through a contract
- Can call multiple contracts

## Calling a contract

A script is a simple program because it consists of a single `main()` function which can:

- Take any number of arguments
- Return a single value of any type

There are two ways to use a script:

- Via the [Rust SDK](https://fuellabs.github.io/fuels-rs/latest/index.html) or [TypeScript SDK](https://fuellabs.github.io/fuels-ts/)
- Manually

The SDKs provide an ergonomic interface for interacting with contracts because they are built to automatically handle various processes however there may be cases where manual use is preferred.

The following example demonstrates the manual implementation which uses the [wallet smart contract](contract.md) to send some asset to a recipient by calling the `send_funds()` function.

```sway
{{#include ../../../code/language/program-types/scripts/transfer/src/main.sw}}
```

Some important points to note are:

1. The return value and parameters are optional
   1. A simple `fn main() { ... }` is sufficient
2. The `abi(<interface>, <b256-address>)` creates a callable type
3. There are optional arguments wrapped in `{}` for the `send_funds()` function
   1. `gas`: a `u64` that represents the gas being forwarded to the contract when it is called
   2. `coins`: a `u64` that represents how many coins are being forwarded with this call
   3. `asset_id`: a `b256` that represents the ID of the _asset type_ of the coins being forwarded
