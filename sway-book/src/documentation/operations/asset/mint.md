# Minting

The [standard library](https://github.com/FuelLabs/sway/tree/master/sway-lib-std) contains a [`module`](https://github.com/FuelLabs/sway/blob/master/sway-lib-std/src/token.sw) that can be used to mint an asset.

There are four functions that can be used to mint:

<!-- no toc -->
- [`mint()`](#mint)
- [`mint_to_address()`](#mint_to_address)
- [`mint_to_contract()`](#mint_to_contract)
- [`mint_to()`](#mint_to)

To use the functions we must import them.

```sway
{{#include ../../../code/operations/asset_operations/src/lib.sw:mint_import}}
```

## `mint()`

To mint some amount of an asset we specify the amount that we would like to mint and pass it into the `mint()` function. 

```sway
{{#include ../../../code/operations/asset_operations/src/lib.sw:mint}}
```

The asset will take the id of the contract that called the mint function.

## `mint_to_address()`

We can [`mint`](#mint) and [`transfer`](transfer/index.md) the asset to an [`address`](../namespace/address.md).

```sway
{{#include ../../../code/operations/asset_operations/src/lib.sw:mint_to_address}}
```

## `mint_to_contract()`

We can [`mint`](#mint) and [`transfer`](transfer/index.md) the asset to a [`contract`](../namespace/contract-id.md).

```sway
{{#include ../../../code/operations/asset_operations/src/lib.sw:mint_to_contract}}
```

## `mint_to()`

We can [`mint`](#mint) and [`transfers`](transfer/contract.md) to an [`Address`](../namespace/address.md) or a [`Contract`](../namespace/contract-id.md).

```sway
{{#include ../../../code/operations/asset_operations/src/lib.sw:mint_to}}
```
