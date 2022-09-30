# ContractId

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
let my_contract_id: ContractId = ~ContractId::from(my_number);
let forty_two: b256 = my_contract_id.into();
```
