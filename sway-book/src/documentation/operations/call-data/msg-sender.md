# Message Sender

To deliver an experience akin to the EVM's access control, the `std` library provides a `msg_sender` function, which identifies a unique caller based upon the call and/or transaction input data.

```sway
{{#include ../../../../examples/msg_sender/src/main.sw}}
```

The `msg_sender` function works as follows:

- If the caller is a contract, then `Result::Ok(Sender)` is returned with the `ContractId` sender variant.
- If the caller is external (i.e. from a script), then all coin input owners in the transaction are checked. If all owners are the same, then `Result::Ok(Sender)` is returned with the `Address` sender variant.
- If the caller is external and coin input owners are different, then the caller cannot be determined and a `Result::Err(AuthError)` is returned.

---

```sway
{{#include ../../../code/operations/call_data/src/lib.sw:msg_sender}}
```
