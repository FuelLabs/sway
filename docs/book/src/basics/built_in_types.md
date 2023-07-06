# Built-in Types

Every value in Sway is of a certain type. Although deep down, all values are just ones and zeroes in the underlying virtual machine, Sway needs to know what those ones and zeroes actually mean. This is accomplished with _types_.

<!-- This section should explain how Sway types are inferred -->
<!-- sway_types:example:start -->
Sway is a statically typed language. At compile time, the types of every value must be known. This does not mean you need to specify every single type: usually, the type can be reasonably inferred by the compiler.
<!-- sway_types:example:end -->

## Primitive Types

<!-- This section should list the primitive types in Sway -->
<!-- prim_types:example:start -->
Sway has the following primitive types:

1. `u8` (8-bit unsigned integer)
1. `u16` (16-bit unsigned integer)
1. `u32` (32-bit unsigned integer)
1. `u64` (64-bit unsigned integer)
1. `str[]` (fixed-length string)
1. `bool` (Boolean `true` or `false`)
1. `b256` (256 bits (32 bytes), i.e. a hash)

All other types in Sway are built up of these primitive types, or references to these primitive types. You may notice that there are no signed integers&mdash;this is by design. In the blockchain domain that Sway occupies, floating-point values and negative numbers have smaller utility, so their implementation has been left up to libraries for specific use cases.
<!-- prim_types:example:end -->

## Numeric Types

All of the unsigned integer types are numeric types.

Numbers can be declared with binary syntax, hexadecimal syntax, base-10 syntax, and underscores for delineation. Let's take a look at the following valid numeric primitives:

```sway
0xffffff    // hexadecimal
0b10101010  // binary
10          // base-10
100_000     // underscore delineated base-10
0x1111_0000 // underscore delineated binary
0xfff_aaa   // underscore delineated hexadecimal
```

<!-- This section should explain the default numeric type in Sway -->
<!-- default_num:example:start -->
The default numeric type is `u64`. The FuelVM's word size is 64 bits, and the cases where using a smaller numeric type saves space are minimal.

If a 64-bit arithmetic operation produces an overflow or an underflow,
computation gets reverted automatically by FuelVM.

8/16/32-bit arithmetic operations are emulated using their 64-bit analogues with
additional overflow/underflow checks inserted, which generally results in
somewhat higher gas consumption.
<!-- default_num:example:end -->

## Boolean Type

<!-- This section should explain the `bool` type -->
<!-- bool:example:start -->
The boolean type (`bool`) has two potential values: `true` or `false`. Boolean values are typically used for conditional logic or validation, for example in `if` expressions. Booleans can be negated, or flipped, with the unary negation operator `!`.
<!-- bool:example:end -->

For example:

```sway
fn returns_false() -> bool {
    let boolean_value: bool = true;
    !boolean_value
}
```

## String Type

<!-- This section should explain the string type in Sway -->
<!-- str:example:start -->
In Sway, static-length strings are a primitive type. This means that when you declare a string, its size is a part of its type. This is necessary for the compiler to know how much memory to give for the storage of that data. The size of the string is denoted with square brackets.
<!-- str:example:end -->

Let's take a look:

```sway
let my_string: str[4] = "fuel";
```

Because the string literal `"fuel"` is four letters, the type is `str[4]`, denoting a static length of 4 characters. Strings default to UTF-8 in Sway.

## Compound Types

_Compound types_ are types that group multiple values into one type. In Sway, we have arrays and tuples.

## Tuple Types

<!-- This section should explain what a tuple is -->
<!-- tuple:example:start -->
A tuple is a general-purpose static-length aggregation of types. In more plain terms, a tuple is a single type that consists of an aggregate of zero or more types. The internal types that make up a tuple, and the tuple's arity, define the tuple's type.
<!-- tuple:example:end -->

Let's take a look at some examples.

```sway
let x: (u64, u64) = (0, 0);
```

This is a tuple, denoted by parenthesized, comma-separated values. Note that the type annotation, `(u64, u64)`, is similar in syntax to the expression which instantiates that type, `(0, 0)`.

```sway
let x: (u64, bool) = (42, true);
assert(x.1);
```

In this example, we have created a new tuple type, `(u64, bool)`, which is a composite of a `u64` and a `bool`.

<!-- This section should explain how to access a value in a tuple -->
<!-- tuple_val:example:start -->
To access a value within a tuple, we use _tuple indexing_: `x.1` stands for the first (zero-indexed, so the `bool`) value of the tuple. Likewise, `x.0` would be the zeroth, `u64` value of the tuple. Tuple values can also be accessed via destructuring.
<!-- tuple_val:example:end -->

```sway
struct Foo {}
let x: (u64, Foo, bool) = (42, Foo {}, true);
let (number, foo, boolean) = x;
```

To create one-arity tuples, we will need to add a trailing comma:

```sway
let x: u64 = (42);     // x is of type u64
let y: (u64) = (42);   // y is of type u64
let z: (u64,) = (42,); // z is of type (u64), i.e. a one-arity tuple
let w: (u64) = (42,);  // type error
```

## Arrays

<!-- This section should explain what an array is -->
<!-- array:example:start -->
An array is similar to a tuple, but an array's values must all be of the same type. Arrays can hold arbitrary types including non-primitive types.
<!-- array:example:end -->

An array is written as a comma-separated list inside square brackets:

```sway
let x = [1, 2, 3, 4, 5];
```

<!-- This section should explain arrays in depth -->
<!-- array_details:example:start -->
Arrays are allocated on the stack since their size is known. An array's size is _always_ static, i.e. it cannot change. An array of five elements cannot become an array of six elements.

Arrays can be iterated over, unlike tuples. An array's type is written as the type the array contains followed by the number of elements, semicolon-separated and within square brackets, e.g. `[u64; 5]`. To access an element in an array, use the _array indexing syntax_, i.e. square brackets.
<!-- array_details:example:end -->

Array elements can also be mutated if the underlying array is declared as mutable:

```sway
let mut x = [1, 2, 3, 4, 5];
x[0] = 0;
```

```sway
{{#include ../../../../examples/arrays/src/main.sw}}
```
