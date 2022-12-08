# Smart Contracts

A smart contract is a piece of bytecode that can be deployed to a blockchain via a [transaction](https://github.com/FuelLabs/fuel-specs/blob/master/src/protocol/tx_format/index.md).

It can be called in the same way that an API may be called to perform computation and store and retrieve data from a database.

A smart contract consists of two parts:

<!--no toc-->
- [Application Binary Interface (`ABI`)](#application-binary-interface-abi)
- [Implementation of the `ABI`](#implementating-the-abi)

## Application Binary Interface (`ABI`)

The `ABI` is a structure which defines the endpoints that a contract exposes for calls. That is to say that functions defined in the `ABI` are considered to be `external` and thus a contract cannot call its own functions.

The following example demonstrates an interface for a wallet which is able to receive and send funds.

The structure begins by using the keyword `abi` followed by the name of the contract.

Inside the declaration are function signatures, annotations denoting the interaction with storage and documentation comments outlining the functionality.

```sway
{{#include ../../../code/language/program-types/contracts/interface/src/lib.sw}}
```

## Implementating the `ABI`

Similar to [traits](https://doc.rust-lang.org/rust-by-example/trait.html) in Rust implementing the `ABI` is done with the syntax `impl <name-of-abi> for Contract`.

All functions defined in the `ABI` must be declared in the implementation.

Since the interface is defined outside of the contract we must import it using the `use` syntax before we can use it.

```sway
{{#include ../../../code/language/program-types/contracts/wallet/src/main.sw}}
```
