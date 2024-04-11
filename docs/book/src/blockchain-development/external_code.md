# External Code Execution

The `std-lib` includes a function called `run_external` that allows Sway contracts to execute arbitrary external Sway code.

This functionality enables features like upgradeable contracts and
proxies.

## Upgradeable Contracts

Upgradeable contracts are designed to allow the logic of a smart contract to be updated after deployment.

Consider this example target contract:

```sway
{{#include ../../../../examples/upgradeable_proxy/target-contract/src/main.sw:target}}
```

This contract has one function called `double_input`, which returns the input value times two.

Below is what an upgradeable proxy contract could look like for this:

```sway
{{#include ../../../../examples/upgradeable_proxy/proxy-contract/src/main.sw:proxy}}
```

The contract has two functions:

- `set_target_contract` updates the `target_contract` variable in storage with the `ContractId` of an external contract.
- `double_input` reads the `target_contract` from storage and uses it to run external code. If the `target_contract` has a function with the same name (`double_input`), the code in the external `double_input` function will run.
In this case, the function will return a `u64`.

## Fallback functions

If the function name doesn't exist in the target contract but a `fallback` function does, the `fallback` function will be triggered.

> If there is no fallback function, the transaction will revert.

You can access function parameters for fallback functions using the `call_frames` module in the `std-lib`.
For example, to access the `_foo` input parameter in the proxy function below, you can use the `second_param` method in the `fallback` function:

```sway
{{#include ../../../../test/src/sdk-harness/test_projects/run_external_proxy/src/main.sw:does_not_exist_in_the_target}}
```

```sway
{{#include ../../../../test/src/sdk-harness/test_projects/run_external_target/src/main.sw:fallback}}
```

In this case, the `does_not_exist_in_the_target` function will return `_foo * 3`.

## How does this differ from calling a contract?

Unlike a normal [contract call](./calling_contracts.md), the context of the contract running
`run_external` is retained for the loaded code.

Additionally, the ABI of the external contract is not required. The proxy contract has no knowledge of the external contract except for its `ContractId`.

## Limitations

Some limitations of `run_external` function are:

- It can only be used with other contracts. Scripts, predicates, and library code cannot be run externally.
- You cannot call an external function that accesses storage in the target contract. For example, if the target contract has a function with a storage annotation such as `#[storage(read)]` or `#[storage(write)]`, the function must be called directly from the target contract instead of the proxy.
