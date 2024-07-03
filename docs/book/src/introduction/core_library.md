# Core Library

The Sway Core Library, like the name suggests contains core operators and logic for the primitive types of the Sway programming language. These traits and methods are an extension of the [primitive types](https://docs.fuel.network/docs/sway/basics/built_in_types/#primitive-types) `u8`, `u16`, `u32`, `u64`, `u256`, `str[]`, `str`, `bool` and , `b256` and can be used where appropriate.

> Please note that Sway Core Library at the time of writing (v0.61.0) does not support operators for unsigned integers of size 128 (`u128`). Although the necessary OP codes exists for operators `u128` in the [FuelVM instruction set](https://docs.fuel.network/docs/specs/fuel-vm/instruction-set/) the team has limited bandwidth preparing for mainnet launch, so please consider [contributing](https://docs.fuel.network/docs/sway/reference/contributing_to_sway/) if this is of interest to you.

The latest core library documentation can be found in the [Core Library Book](https://fuellabs.github.io/sway/master/core/). If the latest version is not compatible please refer to the appropriate tagged release.

## Using the Core Library

Core library functionalities do not need to be explicitly imported and will work out of the box after creating any new Sway project with [`forc new`](../forc/commands/forc_new.md). The `use` keyword is simply not required.

Consider this example of using the modulo function for two like value types:

```sway
let val_1 = 10;
let val_2 = 2;
let result = val_1 % val_2;
```

Developers don't need to concern themselves with underflows and overflows because the Sway compiler automatically manages these issues during execution.

## Core Library Prelude

Sway core operations and logic are limited to their respective types. In other words, intuitively, the add `+` operation will be available for all unsigned integers in Sway but not for booleans.

The prelude contains a list of operations essential to all Sway programs. The latest version of the prelude can be found [here](https://github.com/FuelLabs/sway/blob/master/sway-lib-core/src/prelude.sw).

> In addition to the notice above `Strings` are currently being reworked and do not have essential operations like concatenation etc. Workarounds will be required.

- [`core::primitives::*`](https://github.com/FuelLabs/sway/blob/master/sway-lib-core/src/primitives.sw) a module for getting `max`, `min`, `bits` and `zero`th for integers.
- [`core::primitive_conversions::*`](https://github.com/FuelLabs/sway/blob/master/sway-lib-core/src/primitive_conversions.sw) a module for converting between unsigned integers sizes.
- [`core::raw_ptr::*`](https://github.com/FuelLabs/sway/blob/master/sway-lib-core/src/raw_ptr.sw) a module for dealing with pointers.
- [`core::raw_slice::*`](https://github.com/FuelLabs/sway/blob/master/sway-lib-core/src/raw_slice.sw) a module for converting types to raw slice
- [`core::ops::*`](https://github.com/FuelLabs/sway/blob/master/sway-lib-core/src/ops.sw) a module for operations like `add` or `subtract` and comparisons `equal` and `order`.
- [`core::storage::*`](https://github.com/FuelLabs/sway/blob/master/sway-lib-core/src/storage.sw) a module dealing with storage.
- [`core::str::*`](https://github.com/FuelLabs/sway/blob/master/sway-lib-core/src/str.sw) a module dealing with `str` slices like `len` or converstions like `from_str_array`.
- [`core::codec::*`](https://github.com/FuelLabs/sway/blob/master/sway-lib-core/src/codec.sw) a module to encode and decode data structures.

For the full list of traits and methods available for each primitive type, please refer to the chart below or the [Core Library Book](https://fuellabs.github.io/sway/master/core/index.html).

| Primitive Type                                                                 | Description                      |
| ------------------------------------------------------------------------------ | -------------------------------- |
| [b256](https://fuellabs.github.io/sway/master/core/primitive.b256.html)        | 256 bits (32 bytes), i.e. a hash |
| [bool](https://fuellabs.github.io/sway/master/core/primitive.bool.html)        | Boolean true or false            |
| [str](https://fuellabs.github.io/sway/master/core/primitive.str.html)          | String Slice                     |
| [str[0-63]](https://fuellabs.github.io/sway/master/core/primitive.str[0].html) | Fixed-length string              |
| [u265](https://fuellabs.github.io/sway/master/core/primitive.u256.html)        | 256-bit unsigned integer         |
| [u64](https://fuellabs.github.io/sway/master/core/primitive.u64.html)          | 64-bit unsigned integer          |
| [u32](https://fuellabs.github.io/sway/master/core/primitive.u32.html)          | 32-bit unsigned integer          |
| [u16](https://fuellabs.github.io/sway/master/core/primitive.u16.html)          | 16-bit unsigned integer          |
| [u8](https://fuellabs.github.io/sway/master/core/primitive.u8.html)            | 8-bit unsigned integer           |
