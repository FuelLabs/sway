# Mint to Contract

We can [`mint`](mint.md) and [`transfer`](../transfer/index.md) the asset to a [`Contract`](../../namespace/contract-id.md).

To use the function we must import it.

```sway
{{#include ../../../../code/operations/asset_operations/src/lib.sw:mint_to_import}}
```

To mint some amount of an asset we specify the `amount` that we would like to mint and the `ContractId` to send it to.

```sway
{{#include ../../../../code/operations/asset_operations/src/lib.sw:mint_to_contract}}
