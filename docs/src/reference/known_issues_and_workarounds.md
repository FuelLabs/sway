# Known Issues and Workarounds

## Known Issues

* [#1663](https://github.com/FuelLabs/sway/issues/1663): Using an explicit `return` in all branches of an `if let` expression causes a compile error. The workaround is to use implicit returns instead.

* [#1682](https://github.com/FuelLabs/sway/issues/1682): The compiler currently emits warnings from `sway-lib-std/src/u128.sw` about some unreachable code. Those warnings can be ignored.

* [#1664](https://github.com/FuelLabs/sway/issues/1664): Binary and hex literals cannot be used for integer types (i.e. `u8`, `u16`, `u32`, `u64`). Only decimal literals can be used at the moment.

* [#1657](https://github.com/FuelLabs/sway/issues/1657): Accessing data members of a `struct` directly from a function call does not currently work. The same applies to `enum` types and arrays. The workaround is to store the result of the function call in a temporary variable and accessing the required elements from that variable instead.

* [#1387](https://github.com/FuelLabs/sway/issues/1387): In order to use `unwrap()` from the `result` library, all symbols of `result` needs to be imported via `use::result::*;`.

* [#1665](https://github.com/FuelLabs/sway/issues/1665): Constants defined via the `const` keyword can only have primitive types. That is, it is not possible to define a `ContractId` or an `Address` as `const` for example.

* [#870](https://github.com/FuelLabs/sway/issues/870): All `impl` blocks need to be defined before any of the functions they define can be called.

## General

* Storage variables of types `str[]`, `b256`, `enum`, and arrays are not yet supported. After [this issue](https://github.com/FuelLabs/sway/issues/1229) is closed, it will be possible to read and write these types using [manual storage management](../blockchain-development/storage.md#manual-storage-management). Moreover, storage mappings have to be managed manually for now as shown in the [Subcurrency](../examples/subcurrency.md) example.

* The optimizing pass of the compiler is not yet implemented, therefore bytecode will be more expensive and larger than it would be in production. Note that eventually the optimizer will support zero-cost abstractions, avoiding the need for developers to go down to inline assembly to produce optimal code.

* Currently, we need to parse the Sway code before formatting it. Hence, **the formatter cannot work on Sway code that does not parse correctly**. This requirement may be changed in the future.
