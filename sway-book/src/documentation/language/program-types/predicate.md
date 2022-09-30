# Predicates

A predicate is an executable that does not need to be deployed because it only exists during a transaction. That being said, the predicate hash (specifically the root) is on-chain in the UTXO set.

Similar to a [script](script.md) a predicate consists of a single `main()` function which can take any number of arguments but it must return a Boolean and in order to be valid it must be `true`. 

The Boolean value represents a UTXO spending condition, such as a multisig predicate, and they only allow a subset of the VM instructions to be used (e.g. no jumps).

Unlike scripts, predicates are stateless functions which means that they can neither read from nor write to any contract state.

```sway
{{#include ../../../code/language/program-types/predicates/simple/src/main.sw}}
```
