# Message Sender

The [standard library](https://github.com/FuelLabs/sway/tree/master/sway-lib-std) provides a function [`msg_sender()`](https://github.com/FuelLabs/sway/blob/master/sway-lib-std/src/auth.sw) which retrieves the [Identity](../namespace/identity.md) of the caller.

The identity can be used for a variety of reasons however a common application is access control i.e. restricting functionality for non-privileged users (non-admins).

## Example

To use `msg_sender()` we must import it from the standard library.

```sway
{{#include ../../../code/operations/call_data/src/lib.sw:import_sender}}
```

We can implement access control by specifying that only the owner can call a function.

In the following snippet we accomplish this by comparing the caller `msg_sender()` to the `OWNER`. If a regular user calls the function then it will revert otherwise it will continue to run when called by the `OWNER`.

```sway
{{#include ../../../code/operations/call_data/src/lib.sw:access_control}}
```
