# Native Support for Multiple Asset Types

The FuelVM has built-in support for working with multiple assets.

What does this mean in practice?

As in Ethereum, sending ETH to an address or contract is an operation built in to the FuelVM, meaning it doesn't rely on the existence of some token smart contract to update balances in order to track ownership.

However, unlike Ethereum, the process for sending _any_ native asset is the same. This means that while you would still need a smart contract to handle minting and burning of fungible tokens, the sending and receiving of these tokens can be done independently of the token contract.
