# Wallet

The following example implements a wallet that utilizes the base asset.

## ABI

The [`interface`](../language/program-types/contract.md) contains a function which tracks the amount of the base asset received and a function to transfer the funds. 

```sway
{{#include ../../code/examples/wallet/src/main.sw:abi}}
```

## Implementation

When receiving funds we assert that the wallet accepts the base asset and we track the amount sent. When transfering funds out of the wallet we assert that only the owner can perform the transfer.

```sway
{{#include ../../code/examples/wallet/src/main.sw:implementation}}
```
