# Scripts

A script is a program that can be executed on chain:

- Once to perform some task
- It does not represent ownership of any resources
- It cannot be called by a contract
- Can return a single value of any type

Scripts are state-aware in that while they have no persistent storage (because they only exist during the transaction) they can call contracts and act based upon the returned values and results.

This example script calls a contract:

```sway
script;

use example_contract::MyContract;

struct InputStruct {
    field_1: bool,
    field_2: u64,
}

// All scripts require a main function.
fn main () {
    let caller = abi(MyContract, 0x8900c5bec4ca97d4febf9ceb4754a60d782abbf3cd815836c1872116f203f861);
    let asset_id = 0x7777_7777_7777_7777_7777_7777_7777_7777_7777_7777_7777_7777_7777_7777_7777_7777;
    let input = InputStruct {
        field_1: true,
        field_2: 3,
    };
    caller.foo(5000, 0, asset_id, input);
}
```

Scripts, similar to predicates, rely on a `main()` function as an entry point. You can call other functions defined in a script from the `main()` function or call another contract via an [abi cast](./contract.md#calling-a-smart-contract-from-a-script).

## Scripts and the SDKs

Unlike EVM transactions which can call a contract directly (but can only call a single contract), Fuel transactions execute a script, which may call zero or more contracts. The [Rust](https://github.com/FuelLabs/fuels-rs) and [TypeScript](https://github.com/FuelLabs/fuels-ts) SDKs provide functions to call contract methods as if they were calling contracts directly. Under the hood, the SDKs wrap all contract calls with scripts that contain minimal code to simply make the call and forward script data as call parameters.

## Calling a Smart Contract from a Script

>**Note**: In most cases, calling a contract should be done from the [Rust SDK](../testing/testing-with-rust.md) or the [TypeScript SDK](../frontend/typescript_sdk.md) which provide a more ergonomic UI for interacting with a contract. However, there are situations where manually writing a script to call a contract is required.

Now that we have defined our interface and implemented it for our contract, we need to know how to actually _call_ our contract. Let's take a look at a contract call:

```sway
{{#include ../../../examples/wallet_contract_caller_script/src/main.sw}}
```

The main new concept is the _abi cast_: `abi(AbiName, contract_address)`. This returns a `ContractCaller` type which can be used to call contracts. The methods of the ABI become the methods available on this contract caller: `send_funds` and `receive_funds`. We then directly call the contract ABI method as if it was just a regular method. You also have the option of specifying the following special parameters inside curly braces right before the main list of parameters:

1. `gas`: a `u64` that represents the gas being forwarded to the contract when it is called.
2. `coins`: a `u64` that represents how many coins are being forwarded with this call.
3. `asset_id`: a `b256` that represents the ID of the _asset type_ of the coins being forwarded.

Each special parameter is optional and assumes a default value when skipped:

1. The default value for `gas` is the context gas (i.e. the content of the special register `$cgas`). Refer to the [FuelVM specifications](https://github.com/FuelLabs/fuel-specs/blob/master/specs/vm/main.md) for more information about context gas.
2. The default value for `coins` is 0.
3. The default value for `asset_id` is `ZERO_B256`.
