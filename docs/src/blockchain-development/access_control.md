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

## Contract Ownership

Many contracts require some form of ownership and storing of an `Identity` for access control. When storing an `Identity` where the `BASE_ASSET_ID` should not be allowed as a default(such as ownership), it is recommended to use an `Option<Identity>` instead.

The following is an example of how to properly set ownership of a contract:

```sway
{{#include ../../../examples/ownership/src/main.sw:set_owner_example}}
```

When revoking ownership, it is disadvised to set ownership to the zero address of an `Address` or `ContractId`. Instead, it is recommended to set the ownership `Option<Identity>` to `None`. 

This is an example of how to properly revoke ownership:

```sway
{{#include ../../../examples/ownership/src/main.sw:revoke_owner_example}}
```
