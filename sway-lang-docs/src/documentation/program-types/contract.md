# Smart Contracts

A smart contract is a piece of bytecode that can be deployed to a blockchain via a [transaction](https://github.com/FuelLabs/fuel-specs/blob/master/specs/protocol/tx_format.md).

It can be called in the same way that an API may be called to perform computation and store and retrieve data from a database.

A smart contract consists of two parts:

- [Smart Contracts](#smart-contracts)
  - [Application Binary Interface (`ABI`)](#application-binary-interface-abi)
  - [Implementating the `ABI`](#implementating-the-abi)

## Application Binary Interface (`ABI`)

The `ABI` is a structure which defines the endpoints that a contract exposes for other contracts to call. That is to say that functions defined in the `ABI` are considered to be `external` and thus a contract cannot call its own functions.

The following example demonstrates a simple interface for a wallet which is able to receive and send funds.

The structure begins by using the keyword `abi` followed by the name of the contract. 

Inside the declaration are function signatures, annotations denoting the interaction with storage and documentation comments outlining the functionality.

```sway
{{#include ../../code/wallet/interface/src/lib.sw}}
```

## Implementating the `ABI`

Similar to [traits](https://doc.rust-lang.org/rust-by-example/trait.html) in Rust implementing the `ABI` is done with the syntax `impl <name-of-abi> for Contract`.

All functions defined in the `ABI` must be declared in the implementation.

In the wallet example we import the `Wallet`, with additional standard library imports, declare contract storage for keeping track of the balance and implement two functions:

- `receive_funds()` 
  - Updates the balance only when the specified _BASE_ASSET_ is sent
- `send_funds()`
  - Sends some amount to a recipient if the caller is the owner and the contract has enough of the _BASE_ASSET_

<br>

```sway
{{#include ../../code/wallet/wallet/src/main.sw}}
```
