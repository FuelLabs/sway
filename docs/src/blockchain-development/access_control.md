# Access Control

Smart contracts require the ability to restrict access to and identify certain users or contracts.

Unlike account based blockchains, Fuel calls can originate from an origin which has potentially multiple parties.

## msg_sender

To deliver an experience akin to Ethereum based access control, the `std` library provides a `msg_sender` method, which determines a single sender based upon the call and transaction input data.

```sway
{{#include ../../../examples/msg_sender/src/main.sw}}
```

The `msg_sender` method works as follows: 
- If the caller is a contract, then `Result::Ok(Sender)` is returned with the `ContractId` specified.
- If the caller is external (i.e. from a script), then all transaction input coin owners are checked, if all owners are the same, then `Result::Ok(Sender)` is returned with `Address` specified.
- If input coin owners are different, then the `Sender` cannot be determined and an `Result::Err(AuthError)` is returned.