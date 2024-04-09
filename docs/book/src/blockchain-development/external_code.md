# External Code Execution

The `std-lib` includes a function called `run_external` that allows Sway contracts to execute arbitrary external Sway code.

This functionality enables features like upgradeable contracts and
proxies.

## How does this differ from calling a contract?

Unlike a normal contract call, the context of the contract running
`run_external` is retained for the loaded code.

Additionally, the ABI of the external contract is not required. The proxy contract has no knowledge of the external contract except for its `ContractId`.

## Upgradeable Contracts

Upgradeable contracts are designed to allow the logic of a smart contract to be updated after deployment.

Consider this example target contract:

```sway
{{#include ../../../../examples/upgradeable_proxy/target-contract/src/main.sw:target}}
```

This contract has one function called `double_input`, which returns double of the input value.

Below is what an upgradeable proxy contract could look like for this:

```sway
{{#include ../../../../examples/upgradeable_proxy/proxy-contract/src/main.sw:proxy}}
```

The contract has two functions:

- `set_target_contract` updates the `target_contract` variable in storage with the `ContractId` of a external contract.
- `double_input` reads the `target_contract` from storage and uses it to run external code. If the `target_contract` has a function with the same name (`double_input`), the code in the external `double_input` function will run, and the function will return whatever value is returned from that.

## Fallback functions

If the function name doesn't exist in the target contract, the `fallback` function will be called if the target contract has one.

You can access function parameters for fallback functions using the `call_frames` module in the `std-lib`.

For example, to access the `_foo` input parameter in the proxy function below, you can use the `second_param` method in the `fallback` function:

```sway
{{#include ../../../../test/src/sdk-harness/test_projects/run_external_proxy/src/main.sw:does_not_exist_in_the_target}}
```

```sway
{{#include ../../../../test/src/sdk-harness/test_projects/run_external_target/src/main.sw:fallback}}
```

<!-- a fallback function that takes the `second_param` (the second parameter of the current call frame - in this case it's the input parameter `value`) and returns the value times three. -->

## Limitations

Some limitations of `run_external` function are:

- it can only be used with other contracts. Scripts, predicates, and library code cannot be run.
- you cannot call an external function that accesses storage in the target contract. For example, if the target contract has function with a storage annotation such as `#[storage(read)]` or `#[storage(write)]`, the function must be called directly from the target contract instead of the proxy.
