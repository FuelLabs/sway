# Identity

The `Identity` type is an enum that allows for the handling of both `Address` and `ContractId` types. This is useful in cases where either type is accepted, e.g. receiving funds from an identified sender, but not caring if the sender is an address or a contract.

An `Identity` is implemented as follows.

```sway
{{#include ../../../../sway-lib-std/src/identity.sw:docs_identity}}
```

Casting to an `Identity` must be done explicitly:

```sway
{{#include ../../../../examples/identity/src/main.sw:cast_to_identity}}
```

A `match` statement can be used to return to an `Address` or `ContractId` as well as handle cases in which their execution differs.

```sway
{{#include ../../../../examples/identity/src/main.sw:identity_to_contract_id}}
```

```sway
{{#include ../../../../examples/identity/src/main.sw:different_executions}}
```

A common use case for `Identity` is for access control. The use of `Identity` uniquely allows both `ContractId` and `Address` to have access control inclusively.

```sway
{{#include ../../../../examples/identity/src/main.sw:access_control_with_identity}}
```
