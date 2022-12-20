# To Address or Contract

To use the function we must import it.

```sway
{{#include ../../../../code/operations/asset_operations/src/lib.sw:transfer_import}}
```

To transfer some amount of an asset we specify the `amount` that we would like to transfer, the `asset` and the `Identity` to send it to.

```sway
{{#include ../../../../code/operations/asset_operations/src/lib.sw:transfer}}
```
