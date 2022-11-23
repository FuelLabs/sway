# Scripts

A script is an executable that does not need to be deployed because it only exists during a transaction.

It can be used to replicate the functionality of contracts, such as routers, without the cost of deployment or increase of the blockchain size.

Some properties of a script include:

- It cannot be called by a contract
- It is stateless but can interact with storage through a contract
- Can call multiple contracts

## Example

The following example demonstrates a script which takes one argument and returns the Boolean value of true.

```sway
{{#include ../../../code/language/program-types/scripts/simple/src/main.sw}}
```
