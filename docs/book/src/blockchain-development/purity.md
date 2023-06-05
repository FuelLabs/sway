# Purity

<!-- This section should explain pure functions in Sway -->
<!-- pure:example:start -->
A function is _pure_ if it does not access any [persistent storage](./storage.md). Conversely, the function is _impure_ if it does access any storage. Naturally, as storage is only available in smart contracts, impure functions cannot be used in predicates, scripts, or libraries. A pure function cannot call an impure function.

In Sway, functions are pure by default but can be opted into impurity via the `storage` function attribute. The `storage` attribute may take `read` and/or `write` arguments indicating which type of access the function requires.
<!-- pure:example:end -->

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

> **Note**: the `#[storage(write)]` attribute also permits a function to read from storage. This is due to the fact that partially writing a storage slot requires first reading the slot.

<!-- This section should explain impure functions in Sway -->
<!-- impure:example:start -->
Impure functions which call other impure functions must have at least the same storage privileges or a superset of those for the function called. For example, to call a function with write access a caller must also have write access, or both read and write access. To call a function with read and write access the caller must also have both privileges.
<!-- impure:example:end -->

The `storage` attribute may also be applied to [methods and associated functions](../basics/methods_and_associated_functions.md), [trait](../advanced/traits.md) and [ABI](../sway-program-types/smart_contracts.md#the-abi-declaration) declarations.

<!-- This section should explain the benefits of using pure functions in Sway -->
<!-- pure_benefits:example:start -->
A pure function gives you some guarantees: you will not incur excessive storage gas costs, the compiler can apply additional optimizations, and they are generally easy to reason about and audit.
<!-- pure_benefits:example:end -->

[A similar concept exists in Solidity](https://docs.soliditylang.org/en/v0.8.10/contracts.html#pure-functions). Note that Solidity refers to contract storage as _contract state_, and in the Sway/Fuel ecosystem, these two terms are largely interchangeable.
