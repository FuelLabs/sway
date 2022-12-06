# Predicates

A predicate is an executable that represents a UTXO spending condition, such as a multisig predicate, which has restrictions on the VM instructions that can be used (e.g. no jumps).

It does not need to be deployed to a blockchain because it only exists during a transaction. That being said, the predicate root is on chain as the owner of one or more UTXOs.

Predicates can neither read from nor write to any contract state. Moreover, they cannot use any [contract instructions](https://fuellabs.github.io/fuel-specs/master/vm/instruction_set.html#contract-instructions).

## Transfer Coins to a Predicate

In Fuel, coins can be sent to an address uniquely representing a particular predicate's bytecode (the bytecode root, calculated [here](https://github.com/FuelLabs/fuel-specs/blob/master/src/protocol/id/contract.md)).

## Spending Predicate Coins

The coin UTXOs become spendable not on the provision of a valid signature, but rather if the supplied predicate both has a root that matches their owner, and [evaluates](https://github.com/FuelLabs/fuel-specs/blob/master/src/vm/index.md#predicate-verification) to `true`.

If a predicate reverts, or tries to access impure VM opcodes, the evaluation is automatically `false`.

## Spending Conditions

Predicates may introspect the transaction spending their coins (inputs, outputs, script bytecode, etc.) and may take runtime arguments (the `predicateData`), either or both of which may affect the evaluation of the predicate.

## Example

Similar to a [script](script.md), a predicate consists of a single `main()` function which can take any number of arguments but must return a [Boolean](../built-ins/boolean.md). In order for the predicate to be valid, the returned [Boolean]((../built-ins/boolean.md)) value must be `true`.

```sway
{{#include ../../../code/language/program-types/predicates/simple/src/main.sw}}
```
