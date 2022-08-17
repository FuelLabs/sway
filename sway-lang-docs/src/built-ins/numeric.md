# Numeric Types

All of the unsigned integer types are numeric types, and the `byte` type can also be viewed as an 8-bit unsigned integer.

Numbers can be declared with binary syntax, hexadecimal syntax, base-10 syntax, and underscores for delineation. Let's take a look at the following valid numeric primitives:

```sway
0xffffff    // hexadecimal
0b10101010  // binary
10          // base-10
100_000     // underscore delineated base-10
0x1111_0000 // underscore delineated binary
0xfff_aaa   // underscore delineated hexadecimal
```

The default numeric type is `u64`. The FuelVM's word size is 64 bits, and the cases where using a smaller numeric type saves space are minimal.
