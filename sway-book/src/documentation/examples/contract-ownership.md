# Contract Ownership

The following example implements access control to restrict functionality to a privileged user.

 ## ABI

The [`interface`](../language/program-types/contract.md) contains a function to set the owner and a function that only the owner can use.

```sway
{{#include ../../code/examples/access-control/ownership/src/main.sw:abi}}
```

## Identity

We must keep track of the owner in storage and compare them against the caller via [`msg_sender()`](../operations/call-data/msg-sender.md).

Initially there is no owner so we'll set them to `None`.

```sway
{{#include ../../code/examples/access-control/ownership/src/main.sw:identity}}
```

## Implementation

To set the owner one of two conditions must be met:

- There is no owner
- The current owner is calling the function

To call our `action()` function the caller must be the owner of the contract.

```sway
{{#include ../../code/examples/access-control/ownership/src/main.sw:implementation}}
```
