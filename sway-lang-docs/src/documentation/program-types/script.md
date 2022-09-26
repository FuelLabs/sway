# Scripts

A script is an executable that does not need to be deployed because it only exists during a transaction.

It can be used to replicate the functionality of contracts, such as routers, without the cost of deployment or increase of the blockchain size.

Some properties of a script include:

- It cannot be called by a contract
- It has no persistent storage but can interact with storage through a contract

## Calling a contract

A script is a simple program because it consists of a single `main()` function which can:

- Take any number of arguments
- Return a single value of any type

The following example uses the [wallet smart contract](contract.md) to send some asset to a recipient by calling the `send_funds()` function.

> TODO: fix abi cast link

1. The first step is to declare the type of program which is a `script`
2. In order to call our `Wallet` we must import its interface
3. We declare the parameters that we wish to pass into the script
4. We use an [abi cast](./contract.md#calling-a-smart-contract-from-a-script) to create a callable type for our `Wallet`
5. We call our wallet with optional parameters (wrapped in `{}`) and pass in the parameters
6. We return a boolean to indicate that the call has succeeded

```sway
{{#include ../../code/program-types/scripts/transfer/src/main.sw}}
```

> **Note:**
> The return value is optional and is only included here for demonstration purposes

1. `gas`: a `u64` that represents the gas being forwarded to the contract when it is called
   1. Default: context gas (i.e. the content of the special register `$cgas`).  Refer to the [FuelVM specifications](https://github.com/FuelLabs/fuel-specs/blob/master/specs/vm/main.md)
2. `coins`: a `u64` that represents how many coins are being forwarded with this call
   1. Default: 0
3. `asset_id`: a `b256` that represents the ID of the _asset type_ of the coins being forwarded
   1. Default: `0x000....0` i.e. `b256` of all 0s

>**Note**: In most cases, calling a contract should be done from the [Rust SDK](../testing/testing-with-rust.md) or the [TypeScript SDK](../frontend/typescript_sdk.md) which provide a more ergonomic UI for interacting with a contract. However, there are situations where manually writing a script to call a contract is required.

## Scripts and the SDKs

Unlike EVM transactions which can call a contract directly (but can only call a single contract), Fuel transactions execute a script, which may call zero or more contracts. The [Rust](https://github.com/FuelLabs/fuels-rs) and [TypeScript](https://github.com/FuelLabs/fuels-ts) SDKs provide functions to call contract methods as if they were calling contracts directly. Under the hood, the SDKs wrap all contract calls with scripts that contain minimal code to simply make the call and forward script data as call parameters.
