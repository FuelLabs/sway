# Amount of Asset Sent

The [standard library](https://github.com/FuelLabs/sway/tree/master/sway-lib-std) provides a function [`msg_amount()`](https://github.com/FuelLabs/sway/blob/master/sway-lib-std/src/context.sw) which retrieves the amount of asset sent without any concern for [which asset](msg-asset.md) is sent.

This can be used to set a price or manually track the amount sent by each user.

## Example

To use `msg_amount()` we must import it from the standard library.

```sway
{{#include ../../../code/operations/call_data/src/lib.sw:import_amount}}
```

We can check how much of _any_ asset has been sent and if an incorrect amount has been sent then we may revert.

```sway
{{#include ../../../code/operations/call_data/src/lib.sw:deposit_amount}}
```
