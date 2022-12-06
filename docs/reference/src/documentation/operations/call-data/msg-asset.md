# Asset Sent

The [standard library](https://github.com/FuelLabs/sway/tree/master/sway-lib-std) provides a function [`msg_asset_id()`](https://github.com/FuelLabs/sway/blob/master/sway-lib-std/src/call_frames.sw) which retrieves the [ContractId](../namespace/contract-id.md) of the asset being sent.

This can be used to determine which asset has been sent into the contract.

## Example

To use `msg_asset_id()` we must import it from the standard library. We'll also import the base asset for comparison.

```sway
{{#include ../../../code/operations/call_data/src/lib.sw:import_asset}}
```

We can check which asset has been sent and perform different computation based on the type.

```sway
{{#include ../../../code/operations/call_data/src/lib.sw:deposit}}
```
