# Address

In the UTXO model each output has an address.

The `Address` type is a struct containing a value of a `b256` type. 

```sway
{{#include ../../../code/operations/namespace/src/lib.sw:address}}
```

The value of an `Address` is a hash of either:

- A public key
- [Predicate](../../language/program-types/predicate.md)

The `Address` type is completely separate from a [`ContractId`](contract-id.md) and thus it should not be used when dealing with an address of a deployed contract.

Casting between an `Address` and `b256` can be done in the following way:

```sway
{{#include ../../../code/operations/namespace/src/lib.sw:address_cast}}
```
