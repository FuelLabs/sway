# Mint to Address or Contract

We can [`mint`](./mint.md) and [`transfers`](../transfer/index.md) to an [`Address`](../../namespace/address.md) or a [`Contract`](../../namespace/contract-id.md).

To use the function we must import it.

```sway
{{#include ../../../../code/operations/asset_operations/src/lib.sw:mint_to_import}}
```

To mint some amount of an asset we specify the `amount` that we would like to mint and the `Identity` to send it to.

```sway
{{#include ../../../../code/operations/asset_operations/src/lib.sw:mint_to}}
```
