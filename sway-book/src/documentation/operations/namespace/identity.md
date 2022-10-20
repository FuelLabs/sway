# Identity

The `Identity` type is an enum that allows for the handling of both `Address` and `ContractId` types. This is useful in cases where either type is accepted, e.g. receiving funds from an identified sender, but not caring if the sender is an address or a contract.

An `Identity` is implemented as follows.

```sway
{{#include ../../../../../sway-lib-std/src/identity.sw:docs_identity}}
```

Casting to an `Identity` must be done explicitly:

```sway
{{#include ../../../code/operations/namespace/src/lib.sw:identity_cast}}
```
