# Temporary Workarounds

## Storage Variables and Mappings

Storage variables of types `str[]`, `b256`, `enum`, and arrays are not yet supported. After [this issue](https://github.com/FuelLabs/sway/issues/1229) is closed, it will be possible to read and write these types using [manual storage management](../blockchain-development/storage.md#manual-storage-management). Moreover, storage mappings have to be managed manually for now as shown in the [Subcurrency](../examples/subcurrency.md) example.

## Optimizer

The optimizing pass of the compiler is not yet implemented, therefore bytecode will be more expensive and larger than it would be in production. Note that eventually the optimizer will support zero-cost abstractions, avoiding the need for developers to go down to inline assembly to produce optimal code.

## Formatter

Currently, we need to parse the Sway code before formatting it. Hence, **the formatter cannot work on Sway code that does not parse correctly**. This requirement may be changed in the future.
