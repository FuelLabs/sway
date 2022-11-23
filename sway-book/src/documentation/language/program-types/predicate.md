# Predicates

A predicate is an executable that represents a UTXO spending condition, such as a multisig predicate, which has restrictions on the VM instructions that can be used (e.g. no jumps).

It does not need to be deployed to a blockchain because it only exists during a transaction. That being said, the predicate hash (specifically the root) is on-chain in the UTXO set.

Similar to a [script](script.md), a predicate consists of a single `main()` function which can take any number of arguments but must return a [Boolean](../built-ins/boolean.md). In order for the predicate to be valid, the returned Boolean value must be `true`.

Unlike scripts, predicates are stateless functions which means they can neither read from nor write to any contract state.

## Example

The following example demonstrates a predicate which takes one argument and returns the boolean value of `true`.

```sway
{{#include ../../../code/language/program-types/predicates/simple/src/main.sw}}
```
