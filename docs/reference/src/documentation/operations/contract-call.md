# Call a Contract

A common blockchain operation is communication between [smart contracts](../language/program-types/contract.md).

## Example

To perform a call there are three steps that we must take:

1. Provide an interface to call
2. Create a type that allows us to make a call
3. Call a function on our interface

### Defining the Interface

Let's take the example of a `Vault` to demonstrate how a call can be performed.

```sway
{{#include ../../code/operations/contract_calling/interface/src/lib.sw}}
```

### Creating a Callable Type

To call a function on our `Vault` we must create a type that can perform calls. The syntax for creating a callable type is: `abi(<interface-name>, <b256-address>)`.

### Calling a Contract

The following snippet uses a [`script`](../language/program-types/script.md) to call our `Vault` contract.

```sway
{{#include ../../code/operations/contract_calling/call/src/main.sw}}
```

The `deposit()` function uses pre-defined optional arguments provided by the `Sway` language.
