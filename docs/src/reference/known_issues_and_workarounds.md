# Known Issues and Workarounds

## Known Issues

* [#870](https://github.com/FuelLabs/sway/issues/870): All `impl` blocks need to be defined before any of the functions they define can be called.

## Missing Features

* [#1182](https://github.com/FuelLabs/sway/issues/1182) Arrays in a `storage` block are not yet supported. See the [Manual Storage Management](../blockchain-development/storage.md#manual-storage-management) section for details on how to use `store` and `get` from the standard library to manage storage slots directly. Note, however, that `StorageMap<K, V>` _does_ support arbitrary types for `K` and `V` without any limitations.

* [#428](https://github.com/FuelLabs/sway/issues/428): Arrays are currently immutable which means that changing elements of an array once initialized is not yet possible.

* [#2035](https://github.com/FuelLabs/sway/issues/2035): Dynamic vectors _in storage_ have not yet been implemented. Only [vectors in memory](https://github.com/FuelLabs/sway/blob/master/sway-lib-std/src/vec.sw) are available at the moment.

* [#1188](https://github.com/FuelLabs/sway/issues/1188): Mutable function arguments are not yet allowed except for `self`.

## General

* No compiler optimization passes have been implemented yet, therefore bytecode will be more expensive and larger than it would be in production. Note that eventually the optimizer will support zero-cost abstractions, avoiding the need for developers to go down to inline assembly to produce optimal code.

* Currently, we need to parse the Sway code before formatting it. Hence, **the formatter cannot work on Sway code that does not parse correctly**. This requirement may be changed in the future.
