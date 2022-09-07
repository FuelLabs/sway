# Scripts

A script is runnable bytecode on the chain which executes once to perform some task. It does not represent ownership of any resources and it cannot be called by a contract. A script can return a single value of any type.

Scripts are state-aware in that while they have no persistent storage (because they only exist during the transaction) they can call contracts and act based upon the returned values and results.

This example script calls a contract:

```sway
{{#include ../../../examples/wallet_contract_caller_script/src/main.sw}}
```

Scripts, similar to predicates, rely on a `main()` function as an entry point. You can call other functions defined in a script from the `main()` function or call another contract via an [abi cast](./smart_contracts.md#calling-a-smart-contract-from-a-script).

An example use case for a script would be a router that trades funds through multiple DEXes to get the price for the input asset, or a script to re-adjust a Collateralized Debt Position via a flashloan.

## Scripts and the SDKs

Unlike EVM transactions which can call a contract directly (but can only call a single contract), Fuel transactions execute a script, which may call zero or more contracts. The Rust and TypeScript SDKs provide functions to call contract methods as if they were calling contracts directly. Under the hood, the SDKs wrap all contract calls with scripts that contain minimal code to simply make the call and forward script data as call parameters.
