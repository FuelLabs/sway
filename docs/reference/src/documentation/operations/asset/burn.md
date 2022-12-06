# Burning

Burning an asset means to destroy an asset that a contract has [`minted`](./mint/index.md).

The [standard library](https://github.com/FuelLabs/sway/tree/master/sway-lib-std) contains a [`module`](https://github.com/FuelLabs/sway/blob/master/sway-lib-std/src/token.sw) that can be used to burn an asset.

There is one function used to burn:

<!-- no toc -->
- [`burn()`](#burn)

To use the function we must import it.

```sway
{{#include ../../../code/operations/asset_operations/src/lib.sw:burn_import}}
```

## burn

To burn some amount of an asset we specify the `amount` that we would like to burn and pass it into the `burn()` function.

```sway
{{#include ../../../code/operations/asset_operations/src/lib.sw:burn}}
```
