# Predicates

From the perspective of Sway, predicates are programs that return a Boolean value and which represent ownership of some resource upon execution to true. They have no access to contract storage. Here is a trivial predicate, which always evaluates to true:

```sway
predicate;

// All predicates require a main function which returns a Boolean value.
fn main() -> bool {
    true
}
```

The address of this predicate is `0xd19a5fe4cb9baf41ad9813f1a6fef551107c8e8e3f499a6e32bccbb954a74764`. Any assets sent to this address can be unlocked or claimed by executing the predicate above as it always evaluates to true.

It does not need to be deployed to a blockchain because it only exists during a transaction. That being said, the predicate address is on-chain as the owner of one or more UTXOs.

## Transfer Coins to a Predicate

In Fuel, coins can be sent to a predicate's address(the bytecode root, calculated [here](https://github.com/FuelLabs/fuel-specs/blob/master/src/identifiers/predicate-id.md)).

## Spending Predicate Coins

The coin UTXOs become spendable not on the provision of a valid signature, but rather if the supplied predicate both has a root that matches their owner, and [evaluates](https://github.com/FuelLabs/fuel-specs/blob/master/src/fuel-vm/index.md#predicate-verification) to `true`.

If a predicate reverts, or tries to access impure VM opcodes, the evaluation is automatically `false`.

An analogy for predicates is rather than a traditional 12 or 24 word seed phrase that generates a private key and creates a valid signature, a predicate's code can be viewed as the private key. Anyone with the code may execute a predicate, but only when the predicate evaluates to true may the assets owned by that address be released.

## Spending Conditions

Predicates may introspect the transaction spending their coins (inputs, outputs, script bytecode, etc.) and may take runtime arguments, either or both of which may affect the evaluation of the predicate.

It is important to note that predicates cannot read or write memory. They may however check the inputs and outputs of a transaction. For example in the [OTC Predicate Swap Example](https://github.com/FuelLabs/sway-applications/tree/master/OTC-swap-predicate), a user may specify they would like to swap `asset1` for `asset2` and with amount of `5`. The user would then send `asset1` to the predicate. Only when the predicate can verify that the outputs include `5` coins of `asset2` being sent to the original user, may `asset1` be transferred out of the predicate.
