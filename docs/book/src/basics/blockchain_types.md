# Blockchain Types

Sway is fundamentally a blockchain language, and it offers a selection of types tailored for the blockchain use case.

These are provided via the standard library ([`lib-std`](https://github.com/FuelLabs/sway/tree/master/sway-lib-std)) which both add a degree of type-safety, as well as make the intention of the developer more clear.

## `Address` Type

The `Address` type is a type-safe wrapper around the primitive `b256` type. Unlike the EVM, an address **never** refers to a deployed smart contract (see the `ContractId` type below). An `Address` can be either the hash of a public key (effectively an [externally owned account](https://ethereum.org/en/whitepaper/#ethereum-accounts) if you're coming from the EVM) or the hash of a [predicate](../sway-program-types/predicates.md). Addresses own UTXOs.

An `Address` is implemented as follows.

```sway
pub struct Address {
    value: b256,
}
```

Casting between the `b256` and `Address` types must be done explicitly:

```sway
let my_number: b256 = 0x000000000000000000000000000000000000000000000000000000000000002A;
let my_address: Address = Address::from(my_number);
let forty_two: b256 = my_address.into();
```

## `ContractId` Type

The `ContractId` type is a type-safe wrapper around the primitive `b256` type. A contract's ID is a unique, deterministic identifier analogous to a contract's address in the EVM. Contracts cannot own UTXOs but can own assets.

A `ContractId` is implemented as follows.

```sway
pub struct ContractId {
    value: b256,
}
```

Casting between the `b256` and `ContractId` types must be done explicitly:

```sway
let my_number: b256 = 0x000000000000000000000000000000000000000000000000000000000000002A;
let my_contract_id: ContractId = ContractId::from(my_number);
let forty_two: b256 = my_contract_id.into();
```

## `Identity` Type

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
