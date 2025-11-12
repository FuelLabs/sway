# ContractId

A contract's ID is a unique, deterministic identifier analogous to a contract's address in the EVM. Contracts cannot own UTXOs but they can own assets.

The `ContractId` type is a struct containing a value of a `b256` type.

```sway
{{#include ../../../code/operations/namespace/src/lib.sw:contract_id}}
```

Casting between a `ContractId` and `b256` can be done in the following way:

```sway
{{#include ../../../code/operations/namespace/src/lib.sw:contract_id_cast}}
```
