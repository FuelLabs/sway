# Sway Libraries

The purpose of Sway Libraries is to contain libraries which users can import and use that are not part of the standard library.

These libraries contain helper functions and other tools valuable to blockchain development.

For more information on how to use a Sway-Libs library, please refer to the [Sway-Libs Book](https://fuellabs.github.io/sway-libs/book/getting_started/index.html).

## Assets Libraries

Asset Libraries are any libraries that use [Native Assets](../blockchain-development/native_assets.md) on the Fuel Network.

- [Asset Library](https://fuellabs.github.io/sway-libs/book/asset/index.html); provides helper functions for the [SRC-20](https://github.com/FuelLabs/sway-standards/blob/master/docs/src/src-20-native-asset.md), [SRC-3](https://github.com/FuelLabs/sway-standards/blob/master/docs/src/src-3-minting-and-burning.md), and [SRC-7](https://github.com/FuelLabs/sway-standards/blob/master/docs/src/src-7-asset-metadata.md) standards.

## Access Control and Security Libraries

Access Control and Security Libraries are any libraries that are built and intended to provide additional safety when developing smart contracts.

- [Ownership Library](https://fuellabs.github.io/sway-libs/book/ownership/index.html); used to apply restrictions on functions such that only a **single** user may call them. This library provides helper functions for the [SRC-5; Ownership Standard](https://github.com/FuelLabs/sway-standards/blob/master/docs/src/src-5-ownership.md).
- [Admin Library](https://fuellabs.github.io/sway-libs/book/admin/index.html); used to apply restrictions on functions such that only a select few users may call them like a whitelist.
- [Pausable Library](https://fuellabs.github.io/sway-libs/book/pausable/index.html); allows contracts to implement an emergency stop mechanism.
- [Reentrancy Guard Library](https://fuellabs.github.io/sway-libs/book/reentrancy/index.html); used to detect and prevent reentrancy attacks.

## Cryptography Libraries

Cryptography Libraries are any libraries that provided cryptographic functionality beyond what the std-lib provides.

- [Bytecode Library](https://fuellabs.github.io/sway-libs/book/bytecode/index.html); used for on-chain verification and computation of bytecode roots for contracts and predicates.
- [Merkle Proof Library](https://fuellabs.github.io/sway-libs/book/merkle/index.html); used to verify Binary Merkle Trees computed off-chain.

## Math Libraries

Math Libraries are libraries which provide mathematic functions or number types that are outside of the std-lib's scope.

- [Signed Integers Library](https://fuellabs.github.io/sway-libs/book/signed_integers/index.html); an interface to implement signed integers.

## Data Structures Libraries

Data Structure Libraries are libraries which provide complex data structures which unlock additional functionality for Smart Contracts.

- [Queue Library](https://fuellabs.github.io/sway-libs/book/queue/index.html); a linear data structure that provides First-In-First-Out (FIFO) operations.
