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
