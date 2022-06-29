# Native Support for Multiple Asset Types

The FuelVM has built-in support for working with multiple assets.

What does this mean in practice?

As in the EVM, sending ETH to an address or contract is an operation built into the FuelVM, meaning it doesn't rely on the existence of some token smart contract to update balances to track ownership.

However, unlike the EVM, the process for sending _any_ native asset is the same. This means that while you would still need a smart contract to handle the minting and burning of fungible tokens, the sending and receiving of these tokens can be done independently of the token contract.

## Liquidity Pool Example

All contracts in Fuel can mint and burn their own native token. Contracts can also receive and transfer any native asset including their own. Internal balances of all native assets pushed through calls or minted by the contract are tracked by the FuelVM and can be queried at any point using the balance_of function from the `std` library. Therefore, there is no need for any manual accounting of the contract's balances using persistent storage.

The `std` library provides handy methods for accessing Fuel's native assset operations.

In this example, we show a basic liquidity pool contract minting its own native asset LP token.

```sway
{{#include ../../../examples/liquidity_pool/src/main.sw}}
```

## Native Token Example

In this example, we show a native token contract with more minting, burning and transferring capabilities.

```sway
{{#include ../../../examples/native_token/src/main.sw}}
```
