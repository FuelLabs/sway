# Core Library

The Sway Core Library, like the name suggests contains core operators and logic for the primitive types of the Sway programming language. These traits and methods are an extension of the [primitive types](https://docs.fuel.network/docs/sway/basics/built_in_types/#primitive-types) `u8`, `u16`, `u32`, `u64`, `u256`, `str[]`, `str`, `bool` and , `b256` and can be used where appropriate.

The latest core library documentation can be found [here](https://fuellabs.github.io/sway/master/core/). If the latest version is not compatible please refer to the appropriate tagged release.

## Using the Core Library

Core library functionalities do not need to be explicitly imported and will work out of the box after creating any new Sway project with [`forc new`](../forc/commands/forc_new.md). The `use` keyword is simply not required.

Consider this example of using the modulo function for two like value types:

```sway
let val_1 = 10;
let val_2 = 2;
let result = val_1 % val_2;
```

## Core Library Prelude

The prelude contains a list of operations essential to all Sway programs. The latest version of the prelude can be found [here](https://github.com/FuelLabs/sway/blob/master/sway-lib-core/src/prelude.sw).

- [`core::primitives::*`](https://github.com/FuelLabs/sway/blob/master/sway-lib-core/src/primitives.sw)
- [`core::primitive_conversions::*`](https://github.com/FuelLabs/sway/blob/master/sway-lib-core/src/primitive_conversions.sw)
- [`core::raw_ptr::*`](https://github.com/FuelLabs/sway/blob/master/sway-lib-core/src/raw_ptr.sw)
- [`core::raw_slice::*`](https://github.com/FuelLabs/sway/blob/master/sway-lib-core/src/raw_slice.sw)
- [`core::ops::*`](https://github.com/FuelLabs/sway/blob/master/sway-lib-core/src/ops.sw)
- [`core::storage::*`](https://github.com/FuelLabs/sway/blob/master/sway-lib-core/src/storage.sw)
- [`core::str::*`](https://github.com/FuelLabs/sway/blob/master/sway-lib-core/src/str.sw)
- [`core::codec::*`](https://github.com/FuelLabs/sway/blob/master/sway-lib-core/src/codec.sw)
