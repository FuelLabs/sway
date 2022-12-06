# Purity

A function is _pure_ if it does not access any [persistent storage](./storage.md). Conversely, the function is _impure_ if it does access any storage. Naturally, as storage is only available in smart contracts, impure functions cannot be used in predicates, scripts, or libraries. A pure function cannot call an impure function.

In Sway, functions are pure by default but can be opted into impurity via the `storage` function attribute. The `storage` attribute may take `read` and/or `write` arguments indicating which type of access the function requires.

```sway
#[storage(read)]
fn get_amount() -> u64 {
    ...
}

#[storage(read, write)]
fn increment_amount(increment: u64) -> u64 {
    ...
}
```

Impure functions which call other impure functions must have at least the same storage privileges or a superset of those for the function called. For example, to call a function with write access a caller must also have write access, or both read and write access. To call a function with read and write access the caller must also have both privileges.

The `storage` attribute may also be applied to [methods and associated functions](../basics/methods_and_associated_functions.md), [trait](../advanced/traits.md) and [ABI](../sway-program-types/smart_contracts.md#the-abi-declaration) declarations.

A pure function gives you some guarantees: you will not incur excessive storage gas costs, the compiler can apply additional optimizations, and they are generally easy to reason about and audit. [A similar concept exists in Solidity](https://docs.soliditylang.org/en/v0.8.10/contracts.html#pure-functions). Note that Solidity refers to contract storage as _contract state_, and in the Sway/Fuel ecosystem, these two terms are largely interchangeable.
