# Access Control

Smart contracts require the ability to restrict access to and identify certain users or contracts. Unlike account-based blockchains, transactions in UTXO-based blockchains (i.e. Fuel) do not necessarily have a unique transaction sender. Additional logic is needed to handle this difference, and is provided by the standard library.

## `msg_sender`

To deliver an experience akin to the EVM's access control, the `std` library provides a `msg_sender` function, which identifies a unique caller based upon the call and/or transaction input data.

```sway
{{#include ../../../examples/msg_sender/src/main.sw}}
```

The `msg_sender` function works as follows:

- If the caller is a contract, then `Result::Ok(Sender)` is returned with the `ContractId` sender variant.
- If the caller is external (i.e. from a script), then all coin input owners in the transaction are checked. If all owners are the same, then `Result::Ok(Sender)` is returned with the `Address` sender variant.
- If the caller is external and coin input owners are different, then the caller cannot be determined and a `Result::Err(AuthError)` is returned.

## Using `BASE_ASSET_ID` as default/null address

The `BASE_ASSET_ID` constant is **not** a reserved address and should not be treated as such.

`BASE_ASSET_ID` is defined in the standard library as a `ContractId` type whose value should not be depended on.

Many contracts require some form of ownership. When revoking ownership, it is disadvised to use the `BASE_ASSET_ID` of a `Address` or `ContractId` to reset the owner. Instead, it is recommended to use an `Option` of type `Identity` and setting the `Option` to `None`. 

Here is an example of how to properly revoke ownership:

```sway
{{#include ../../../examples/ownership/src/main.sw:revoke_owner_example}}
```
