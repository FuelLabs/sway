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
   5. `hexadecimal`, `binary` & `base-10` syntax
2. [Boolean](boolean.md)
   1. `bool` (true or false)
3. [Strings](string.md)
   1. `str[]` (fixed-length string)
4. [Bytes](b256.md)
   1. `b256` (256 bits / 32 bytes, i.e. a hash)

All other types in Sway are built up of these primitive types, or references to these primitive types. 

You may notice that there are no signed integers - this is by design. In the blockchain domain that Sway occupies, floating-point values and negative numbers have smaller utility, so their implementation has been left up to libraries for specific use cases.

> The default numeric type is `u64`. The FuelVM's word size is 64 bits, and the cases where using a smaller numeric type saves space are minimal.

## Compound Types

Compound types are types that group multiple values into one type. 

Sway has the following compound types:

1. [Arrays](arrays.md)
2. [Tuples](tuples.md)
3. [Structs](structs.md)
4. [Enums](enums.md)
