# Primitive Types

Sway has the following primitive types:

1. `u8` (8-bit unsigned integer)
1. `u16` (16-bit unsigned integer)
1. `u32` (32-bit unsigned integer)
1. `u64` (64-bit unsigned integer)
1. `str[]` (fixed-length string)
1. `bool` (Boolean `true` or `false`)
1. `b256` (256 bits (32 bytes), i.e. a hash)

All other types in Sway are built up of these primitive types, or references to these primitive types. You may notice that there are no signed integers&mdash;this is by design. In the blockchain domain that Sway occupies, floating-point values and negative numbers have smaller utility, so their implementation has been left up to libraries for specific use cases.
