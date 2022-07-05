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

Many contracts require some form of ownership for access control. To accomplish this, it is recommended that a storage variable of type `Option<Identity>` is used to keep track of the owner. This allows setting and revoking ownership using the variants `Some(..)` and `None` respectively. This is better, safer, and more readable than using the `Identity` type directly where revoking ownership has to be done using some magic value such as `std::constants::ZERO_B256` or otherwise.

The following is an example of how to properly set ownership of a contract:

```sway
{{#include ../../../examples/ownership/src/main.sw:set_owner_example}}
```

The following is an example of how to properly revoke ownership of a contract:

```sway
{{#include ../../../examples/ownership/src/main.sw:revoke_owner_example}}
```
