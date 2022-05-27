# Subcurrency

The following is a simple example of a subcurrency which implements functionality to mint and send token. It is a ledger-based token, i.e. the contract maintains a ledger of user account balances.

Being a ledger-based token, this example does not use Fuel's [native asset system](../blockchain-development/native_assets.md). It is not recommended to actually use ledger-based tokens in production; this example is here purely for illustrative purposes.

```sway
{{#include ../../../examples/subcurrency/src/main.sw:body}}
```
