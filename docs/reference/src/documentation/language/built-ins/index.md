# Built-in Types

Sway is a statically typed language therefore every value must be known at compile time. This means that each value must have a _type_ and the compiler can _usually_ infer the type without the user being required to specify it.

Sway provides a number of out-of-the-box (primitive) types which can be used to construct more complex data structures and programs.

## Primitive Types

Sway has the following primitive types:

1. [Numerics](numeric.md)
   1. `u8` (8-bit unsigned integer)
   2. `u16` (16-bit unsigned integer)
   3. `u32` (32-bit unsigned integer)
   4. `u64` (64-bit unsigned integer)
   5. `u256` (256-bit unsigned integer)
   6. `hexadecimal`, `binary` & `base-10` syntax
2. [Boolean](boolean.md)
   1. `bool` (true or false)
3. [Strings](string.md)
   1. `str` (string slice)
   1. `str[n]` (fixed-length string of size n)
4. [Bytes](b256.md)
   1. `b256` (256 bits / 32 bytes, i.e. a hash)
5. [Slices](slices.md)

<!-- TODO: The following sentence does not belong here. We need to convey the default size, including word size, somewhere however not on this page -->

The default numeric type is `u64`. The FuelVM's word size is 64 bits, and the cases where using a smaller numeric type to save space are minimal.

All other types in Sway are built up of these primitive types, or references to these primitive types.

## Compound Types

Compound types are types that group multiple values into one type.

Sway has the following compound types:

1. [Arrays](arrays.md)
2. [Tuples](tuples.md)
3. [Structs](structs.md)
4. [Enums](enums.md)
