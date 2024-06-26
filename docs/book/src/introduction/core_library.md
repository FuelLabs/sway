# Core Library

The Sway Core Library, like the name suggests contains core operators and logic for the primitive types of the Sway programming language. These traits and methods are an extension of the [primitive types](https://docs.fuel.network/docs/sway/basics/built_in_types/#primitive-types) `u8`, `u16`, `u32`, `u64`, `u256`, `str[]`, `str`, `bool` and , `b256` and can be used where appropriate. 
> Please note that Sway Core Library at the time of writing (v0.61.0) does not support operators for unsigned integers of size 128 (`u128`). Although the necessary OP codes exists for operators `u128` in the [FuelVM instruction set](https://docs.fuel.network/docs/specs/fuel-vm/instruction-set/) the team has limited bandwidth preparing for mainnet launch, so please consider [contributing](https://docs.fuel.network/docs/sway/reference/contributing_to_sway/) if this is of interest to you.

The latest core library documentation can be found in the [Core Library Book](https://fuellabs.github.io/sway/master/core/). If the latest version is not compatible please refer to the appropriate tagged release.

## Using the Core Library
Unlike their standard library counterparts, core library functionalities do not need to be explicitly imported and will work out of the box after creating any new Sway project with `forc new`. The `use` keyword is simply not required.

Consider this example of using the modulo function for two like value types in a `struct`:
```sway
let struct1 = MyStruct { val: 10 };
let struct2 = MyStruct { val: 2 };
let result_struct = struct1 % struct2;
```

A bonus of developing with Sway is that developers do not have to worry about underflows and overflows, as these are handled by the FuelVM by design.

## Core Library Prelude

Sway core operations and logic are limited to their respective types. In other words, intuitively, the add `+` operation will be available for all unsigned integers in Sway but not for booleans.

The prelude contains a list of operations essential to all Sway programs. The latest version of the prelude can be found [here](https://github.com/FuelLabs/sway/blob/master/sway-lib-core/src/prelude.sw).

> In addition to the notice above `Strings` are currently being reworked and do not have essential operations like concatenation etc. Workarounds will be required.

### Primitives
`max()` The largest value that can be represented by this integer type i.e. `u256::max()`
`min()` The smallest value that can be represented by this integer type i.e. `u256::min()`
`bits()` The size of this integer type in bits i.e. `u256::bits()`
`zero()` the zero value for this integer type i.e. `u256::zero()`

### Primitive Conversions
`as_u256` Converts any unsigned integer smaller than `u256` including `b256` to a `u256` i.e. `val.as_u256()`
`as_u64`  Converts any unsigned integer smaller than `u64` to a `u64` i.e. `val.as_u64()`
`as_u32` Converts any unsigned integer smaller than `u32` to a `u32` i.e. `val.as_u32()`
`as_b256` Converts a `u256` to a `b256` i.e. `val.as_b256()`

### Operations
`add` Add two values of the same type i.e. `let res = val1 + val2`
`subtract` Subtract two values of the same type i.e. `let res = val1 - val2`
`multiply` Multiply two values of the same type i.e. `let res = val1 * val2`
`divide` Divide two values of the same type i.e. `let res = val1 * val2`
`modulo` Modulo two values of the same type i.e. `let res = val1 % val2`
`not` Inverts the value of the type i.e. `let res = !val`
`equal` Evaluates if two values of the same type are equal i.e. `let res = val1 == val2` or `let res = val1 != val2`
`order` Evaluates if one value of the same type is greater than another i.e. `let res = val1 > val2` or `let res = val1 >= val2`
`shift` Bit shift left by an amount i.e. `let res = val1 >> 1` or `let res = val1 << 1`

### String
`len` Return the length of the string slice in bytes i.e. `let res = val.len()`
`as_ptr` Return a `raw_ptr` to the beginning of the string slice on the heap i.e. `let res = val.as_ptr`
`from_str_array` Convert a string array to string i.e. `let res: str = from_str_array(val)`

### Storage
`slot` The assigned location in storage i.e. `let res = val.slot()`
`offset` The assigned offset based on the data structure `T` i.e. `let res = val.offset()`
`field_id` A unique identifier i.e. `let res = val.field_id()`

### Raw Slice
`slice` Converts self into a `raw_slice` i.e. `let slice = my_type.as_raw_slice()`

### Codec
`abi_encode` Encodes a value based on the buffer i.e. `let res = val.abi_encode(buffer)`
`abi_decode` Decodes a type based on the buffer i.e. `let res = my_type::abi_decode(buffer)`

For the full list of traits and methods available for each primitive type, please refer to the chart below or the [Core Library Book](https://fuellabs.github.io/sway/master/core/index.html).

| Primitive Type     | Description                      |
|--------------------|----------------------------------|
| [b256](https://fuellabs.github.io/sway/master/core/primitive.b256.html)           | 256 bits (32 bytes), i.e. a hash |
| [bool](https://fuellabs.github.io/sway/master/core/primitive.bool.html)           | Boolean true or false            |
| [str](https://fuellabs.github.io/sway/master/core/primitive.str.html)            | String Slice                     |
| [str[0-63]](https://fuellabs.github.io/sway/master/core/primitive.str[0].html)      | Fixed-length string              |
| [u265](https://fuellabs.github.io/sway/master/core/primitive.u256.html)           | 256-bit unsigned integer         |
| [u64](https://fuellabs.github.io/sway/master/core/primitive.u64.html)            | 64-bit unsigned integer          |
| [u32](https://fuellabs.github.io/sway/master/core/primitive.u32.html)            | 32-bit unsigned integer          |
| [u16](https://fuellabs.github.io/sway/master/core/primitive.u16.html)            | 16-bit unsigned integer          |
| [u8](https://fuellabs.github.io/sway/master/core/primitive.u8.html)             | 8-bit unsigned integer           |

 