# Known Issues and Workarounds

## Known Issues

* [#870](https://github.com/FuelLabs/sway/issues/870): All `impl` blocks need to be defined before any of the functions they define can be called.  This includes sibling functions in the same `impl` declaration, i.e., functions in an `impl` can't call each other yet.

## Missing Features

* [#1182](https://github.com/FuelLabs/sway/issues/1182) Arrays in a `storage` block are not yet supported. See the [Manual Storage Management](../blockchain-development/storage.md#manual-storage-management) section for details on how to use `store` and `get` from the standard library to manage storage slots directly. Note, however, that `StorageMap<K, V>` _does_ support arbitrary types for `K` and `V` without any limitations.

## General

* No compiler optimization passes have been implemented yet, therefore bytecode will be more expensive and larger than it would be in production. Note that eventually the optimizer will support zero-cost abstractions, avoiding the need for developers to go down to inline assembly to produce optimal code.
