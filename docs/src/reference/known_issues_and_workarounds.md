# Known Issues and Workarounds

## Known Issues

* [#870](https://github.com/FuelLabs/sway/issues/870): All `impl` blocks need to be defined before any of the functions they define can be called.  This includes sibling functions in the same `impl` declaration, i.e., functions in an `impl` can't call each other yet.

## Missing Features

* [#1182](https://github.com/FuelLabs/sway/issues/1182) Arrays in a `storage` block are not yet supported. See the [Manual Storage Management](../blockchain-development/storage.md#manual-storage-management) section for details on how to use `store` and `get` from the standard library to manage storage slots directly. Note, however, that `StorageMap<K, V>` _does_ support arbitrary types for `K` and `V` without any limitations.

* [#428](https://github.com/FuelLabs/sway/issues/428): Arrays are currently immutable which means that changing elements of an array once initialized is not yet possible.

* [#1796](https://github.com/FuelLabs/sway/issues/2465): It is not yet allowed to use `StorageMap<K, V>` as a component of a complex type such as a struct or an enum.

* [#2647](https://github.com/FuelLabs/sway/issues/2647): Currently, it is only possible to define configuration-time constants that have [primitive types](built_in_types.md#primitive-types) and that are initialized using literals.

## General

* No compiler optimization passes have been implemented yet, therefore bytecode will be more expensive and larger than it would be in production. Note that eventually the optimizer will support zero-cost abstractions, avoiding the need for developers to go down to inline assembly to produce optimal code.

* Currently, we need to parse the Sway code before formatting it. Hence, **the formatter cannot work on Sway code that does not parse correctly**. This requirement may be changed in the future.
