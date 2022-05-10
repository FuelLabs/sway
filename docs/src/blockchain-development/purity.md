# Purity

A function is _pure_ if it does not access any [persistent storage](./storage.md). Conversely, the function is _impure_ if it does access any storage. Naturally, as storage is only available in smart contracts, impure functions cannot be used in predicates, scripts, or libraries. A pure function cannot call an impure function.

In Sway, functions are pure by default but can be opted in to impurity via function annotations which are currently work-in-progress.

A pure function gives you some guarantees: you will not incur excessive storage gas costs, the compiler can apply additional optimizations, and they are generally easy to reason about and audit. [A similar concept exists in Solidity](https://docs.soliditylang.org/en/v0.8.10/contracts.html#pure-functions). Note that Solidity refers to contract storage as _contract state_, and in the Sway/Fuel ecosystem, these two terms are largely interchangeable.
