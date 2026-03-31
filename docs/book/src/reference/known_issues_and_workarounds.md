# Known Issues and Workarounds

## Known Issues

* [#870](https://github.com/FuelLabs/sway/issues/870): All `impl` blocks need to be defined before any of the functions they define can be called.  This includes sibling functions in the same `impl` declaration, i.e., functions in an `impl` can't call each other yet.

## Missing Features

* [#1182](https://github.com/FuelLabs/sway/issues/1182) Arrays in a `storage` block are not yet supported. See the [Manual Storage Management](../advanced/advanced_storage.md#manual-storage-management) section for details on how to use `store` and `get` from the standard library to manage storage slots directly. Note, however, that `StorageMap<K, V>` _does_ support arbitrary types for `K` and `V` without any limitations.


## Importing

In [external libraries](../../language/program-types/libraries/external.md) we have looked at how a library can be imported into a project so that code can be reused.

When it comes to importing only external libraries can be imported through the `Forc.toml` file; any other type of program will result in an error.

This means that the following projects cannot be imported:

- [contracts](../../language/program-types/contract.md)
- [internal libraries](../../language/program-types/libraries/internal.md)
- [scripts](../../language/program-types/script.md)
- [predicates](../../language/program-types/predicate.md)

While contracts cannot be imported, a workaround is to move the contract's `abi` declaration into an [external library](../../language/program-types/libraries/external.md) and import that library anywhere the ABI is needed.

## Pattern Matching

### Nested Match Expressions

In [nested match expressions](../../language/control-flow/match/complex/nested-expression.md) we nest a `match` expression by embedding it inside the `{}` brackets on the right side of the arrow `=>`.

Match expressions cannot be used as a pattern, the left side of the arrow `=>`.

### Constants

When matching on [constants](../../language/control-flow/match/complex/constant.md) we specify that a constant must be used in order to match on a variable. Dynamic values, such as an argument to a function, cannot be matched upon because it will be treated as a [`catch_all`](../../language/control-flow/match/single-line.md) case and thus any subsequent patterns will not be checked.


## General

* No compiler optimization passes have been implemented yet, therefore bytecode will be more expensive and larger than it would be in production. Note that eventually the optimizer will support zero-cost abstractions, avoiding the need for developers to go down to inline assembly to produce optimal code.
