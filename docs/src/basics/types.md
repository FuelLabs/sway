# Types

Every value in Sway is of a certain type. Although, deep down, all values are just ones and zeroes in silicon, Sway needs to know what those ones and zeroes actually mean. This is accomplished with _types_.

Sway is a statically typed language. At compile time, the types of every value must be known. This does not mean you need to specify every single type: usually the type can be reasonably inferred.

## Primitive Types

Sway has the following primitive types:

1. u8 (8-bit unsigned integer)
1. u16 (16-bit unsigned integer)
1. u32 (32-bit unsigned integer)
1. u64 (64-bit unsigned integer)
1. String
1. Boolean
1. Byte
1. b256 (256 bits(32 bytes) -- i.e. a hash)
1. Static-length arrays (as of now, not yet implemented)

All other types in Sway are built up of these primitive types, or references to these primitive types. You may notice that there are no signed integers -- this is by design. In the blockchain domain that Sway occupies, floating point values and negative numbers have smaller utility, so their implementation has been left up to libraries for specific use cases.

## Numeric Types

All of the unsigned integer types are numeric types, and the `byte` type can also be viewed as an 8-bit unsigned integer.

Numbers can be declared with binary syntax, hexadecimal syntax, base-10 syntax, and with underscores for delineation. Let's take a look at the following valid numeric primitives:

```sway
0xffffff    // hexadecimal
0b10101010  // binary
10          // base-10
100_000     // underscore delineated base-10
0x1111_0000 // underscore delineated binary
0xfff_aaa   // underscore delineated binary
```

The default numeric type is `u64`. The FuelVM's word size is 64 bits, and the cases where using a smaller numeric type saves space are minimal.

## Boolean Type

The boolean type (`bool`) has two potential values: `true` or `false`. Boolean values are typically used for conditional logic or validation, for example in `if` expressions. Booleans can be negated, or flipped, with the unary negation operator `!`. For example:

```sway
fn returns_false() -> bool {
  let boolean_value: bool = true;
  !boolean_value
}
```

## String Type

In Sway, static-length strings are a primitive type. This means that when you declare a string, its size is a part of its type. This is necessary for the compiler to know how much memory to give for storage of that data. The size of the string is denoted with square brackets. Let's take a look:

```sway
let my_string: str[4] = "fuel";
```

Because the string literal `"fuel"` is four letters, the type is `str[4]`, denoting a static length of 4 characters. Strings default to UTF-8 in Sway.

## Address type

In Sway, the Address type is a type-safe wrapper around the primitive b256 type.
(as of now, not yet implemented)
