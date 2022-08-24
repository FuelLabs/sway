# Primitive Types

Primitive types are types that are build into the language. These types can also be basic data structures that are used to build other types.

Sway has the following primitive types:

1. Integers
   1. `u8` (8-bit unsigned integer)
   2. `u16` (16-bit unsigned integer)
   3. `u32` (32-bit unsigned integer)
   4. `u64` (64-bit unsigned integer)
2. Strings
   1. `str[]` (fixed-length string)
3. Boolean
   1. `bool` (true or false)
4. Bytes
   1. `b256` (256 bits / 32 bytes, i.e. a hash)

All other types in Sway are built up of these primitive types, or references to these primitive types. 

You may notice that there are no signed integers - this is by design. In the blockchain domain that Sway occupies, floating-point values and negative numbers have smaller utility, so their implementation has been left up to libraries for specific use cases.
