# Mint to Address

We can [`mint`](mint.md) and [`transfer`](../transfer/index.md) the asset to an [`Address`](../../namespace/address.md).

To use the function we must import it.

```sway
{{#include ../../../../code/operations/asset_operations/src/lib.sw:mint_to_address_import}}
```

To mint some amount of an asset we specify the `amount` that we would like to mint and the `Address` to send it to.

```sway
{{#include ../../../../code/operations/asset_operations/src/lib.sw:mint_to_address}}
```
