# Minting

Minting an asset means to create a new asset with an id of the contract that created it.

The [standard library](https://github.com/FuelLabs/sway/tree/master/sway-lib-std) contains a [`module`](https://github.com/FuelLabs/sway/blob/master/sway-lib-std/src/token.sw) that can be used to mint an asset.

There are four functions that can be used to mint:

<!-- no toc -->
- [`mint()`](./mint.md)
- [`mint_to_address()`](./address.md)
- [`mint_to_contract()`](./contract.md)
- [`mint_to()`](./address-or-contract.md)
