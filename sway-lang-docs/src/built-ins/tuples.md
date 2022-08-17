# Tuples

A tuple is a general-purpose static-length aggregation of types. In more plain terms, a tuple is a single type that consists of an aggregate of zero or more types. The internal types that make up a tuple, and the tuple's arity, define the tuple's type. Let's take a look at some examples.

```sway
let x: (u64, u64) = (0, 0);
```

This is a tuple, denoted by parenthesized, comma-separated values. Note that the type annotation, `(u64, u64)`, is similar in syntax to the expression which instantiates that type, `(0, 0)`.

```sway
let x: (u64, bool) = (42, true);
assert(x.1);
```

In this example, we have created a new tuple type, `(u64, bool)`, which is a composite of a `u64` and a `bool`. To access a value within a tuple, we use _tuple indexing_: `x.1` stands for the first (zero-indexed, so the `bool`) value of the tuple. Likewise, `x.0` would be the zeroth, `u64` value of the tuple. Tuple values can also be accessed via destructuring:

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

Tuples are a [basic static-length type](./built_in_types.md#tuple-types) which contain multiple different types within themselves. The type of a tuple is defined by the types of the values within it, and a tuple can contain basic types as well as structs and enums.

You can access values directly by using the `.` syntax. Moreover, multiple variables can be extracted from a tuple using the destructuring syntax.

```sway
{{#include ../../../examples/tuples/src/main.sw}}
```
