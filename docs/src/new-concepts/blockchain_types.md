# Blockchain Types

Sway has a selection of types provided via the standard library (`lib-std`) which both add a degree of type-safety, as well as make the intention of the developer more clear.

## Address type

The `Address` type is a type-safe wrapper around the primitive `b256` type. Unlike Ethereum, an address **never** refers to a deployed smart contract (see the `ContractId` type below). An `Address` can be either the hash of a public key (An Externally Owned Address if you're coming from Ethereum) or the hash of a [predicate](../sway-on-chain/predicates.md).

```sway
pub struct Address {
    value: b256,
}
```

Casting between the `b256` & `Address` types must be done explicitly:

```sway
let my_number: b256 = 0x000000000000000000000000000000000000000000000000000000000000002A;
let my_address: Address = ~Address::from(my_number);
let forty_two: b256 = my_address.into();
```

## ContractId type

The `ContractId` type is a type-safe wrapper around the primitive `b256` type. A contract's id is a unique, deterministic identifier analogous to a contract's address on Ethereum.

```sway
pub struct ContractId {
    value: b256,
}
```

Casting between the `b256` & `ContractId` types must be done explicitly:

```sway
let my_number: b256 = 0x000000000000000000000000000000000000000000000000000000000000002A;
let my_contract_id: ContractId = ~ContractId::from(my_number);
let forty_two: b256 = my_contract_id.into();
```
