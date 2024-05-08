# External Code Execution

The `std-lib` includes a function called `run_external` that allows Sway contracts to execute arbitrary external Sway code.

This functionality enables features like upgradeable contracts and
proxies.

## Upgradeable Contracts

Upgradeable contracts are designed to allow the logic of a smart contract to be updated after deployment.

Consider this example proxy contract:

```sway
{{#include ../../../../examples/upgradeable_proxy/proxy/src/main.sw:proxy}}
```

The contract has two functions:

- `set_target_contract` updates the `target_contract` variable in storage with the `ContractId` of an external contract.
- `double_input` reads the `target_contract` from storage and uses it to run external code. If the `target_contract` has a function with the same name (`double_input`), the code in the external `double_input` function will run.
In this case, the function will return a `u64`.

Notice in the `Proxy` example above, the storage block has a `namespace` attribute. Using this attribute is considered a best practice for all proxy contracts in Sway, because it will prevent storage collisions with the implementation contract, as the implementation contract has access to both storage contexts.

Below is what an implementation contract could look like for this:

```sway
{{#include ../../../../examples/upgradeable_proxy/implementation/src/main.sw:target}}
```

This contract has one function called `double_input`, which calculates the input value times two, updates the `value` variable in storage, and returns the new value.

## How does this differ from calling a contract?

There are a couple of major differences between calling a contract directly and using the `run_external` method.

First, to use `run_external`, the ABI of the external contract is not required. The proxy contract has no knowledge of the external contract except for its `ContractId`.

### Upgradeable Contract Storage

Second, the storage context of the proxy contract is retained for the loaded code.
This means that in the examples above, the `value` variable gets updated in the storage for the *proxy* contract.

For example, if you were to read the `value` variable by directly calling the implementation contract, you would get a different result than if you read it through the proxy contract.
The proxy contract loads the code and executes it in its own context.

## Fallback functions

If the function name doesn't exist in the target contract but a `fallback` function does, the `fallback` function will be triggered.

> If there is no fallback function, the transaction will revert.

You can access function parameters for fallback functions using the `call_frames` module in the `std-lib`.
For example, to access the `_foo` input parameter in the proxy function below, you can use the `called_args` method in the `fallback` function:

```sway
{{#include ../../../../test/src/sdk-harness/test_projects/run_external_proxy/src/main.sw:does_not_exist_in_the_target}}
```

```sway
{{#include ../../../../test/src/sdk-harness/test_projects/run_external_target/src/main.sw:fallback}}
```

In this case, the `does_not_exist_in_the_target` function will return `_foo * 3`.

## Limitations

Some limitations of `run_external` function are:

- It can only be used with other contracts. Scripts, predicates, and library code cannot be run externally.
- If you change the implementation contract, you must maintain the same order of previous storage variables and types, as this is what has been stored in the proxy storage.
- You can't use the call stack in another call frame before you use `run_external`. You can only use the call stack within the call frame that contains `run_external`.
