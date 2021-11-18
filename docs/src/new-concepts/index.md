# Concepts

## Addresses & Contract-IDs

Addresses in Sway are similar to Ethereum addresses. The 2 differences are that Sway addresses are 32 bytes long (instead of 20) and are computed with Sha256(PublicKey) (instead of using Keccak256).

Contracts, on the other hand, have a `contract_id` rather than an address.
A contract's id is also 32 bytes long, and is calculated with:
`sha256(0x4655454C ++ tx.data.salt ++ root(tx.data.witnesses[bytecodeWitnessIndex].data))`

## Native support for different token types

The FuelVM has built-in support for working with tokens other than Ether!

What does this mean in practice?
Well as in Ethereum, sending Eth to an address or contract is an operation built in to the FuelVM, meaning it doesn't rely on the existence of some token smart contract to update balances in order to track ownership.

However, unlike Ethereum, the process for sending __any__ token is the same! This means that while you would still need a smart contract to handle minting & burning of tokens (FTs or NFTs), the sending and receiving of these tokens can be done independently of the token contract.

## UTXOs

Unlike the Account-based model used by Ethereum, Fuel leverages a model based on Unspent Transaction Outputs(UTXOs).

__TODO... !!!__
