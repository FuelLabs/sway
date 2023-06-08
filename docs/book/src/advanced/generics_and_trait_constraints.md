# Generics and Trait Constraints

## Generics as Constraints

At a high level, Sway allows you to define constraints, or restrictions, that
allow you to strike a balance between writing abstract and reusable code and
enforcing compile-time checks to determine if the abstract code that you've
written is correct.

The "abstract and reusable" part largely comes from [generic types](./generic_types.md) and the
"enforcing compile-time checks" part largely comes from trait constraints.
Generic types can be used with functions, structs, and enums (as we have seen in
this book), but they can also be used with traits.

## Generic Traits

Combining generic types with traits allows you to write abstract and reusable
traits that can be implemented for any number of data types.

For example, imagine that you want to write a trait for converting between
different types. This would be similar to Rust's `Into` and `From` traits. In
Sway your conversion trait would look something like:

```sway
{{#include ../../../../examples/traits/src/main.sw:trait_definition}}
```

The trait `Convert` takes a generic type `T`. `Convert` has one method
`from`, which takes one parameter of type `T` and returns a `Self`. This means
that when you implement `Convert` for a data type, `from` will return the type
of that data type but will take as input the type that you define as `T`. Here
is an example:

```sway
{{#include ../../../../examples/traits/src/main.sw:trait_impl}}
```

In this example, you have two different data types, `Square` and `Rectangle`.
You know that all squares are rectangles and thus `Square` can convert into `Rectangle` (but not vice
versa) and thus you can implement the conversion trait for those types.

If we want to call these methods we can do so by:

```sway
{{#include ../../../../examples/traits/src/main.sw:trait_usage}}
```

## Trait Constraints

Trait constraints allow you to use generic types and traits to place constraints
on what abstract code you are willing to accept in your program as correct.
These constraints take the form of compile-time checks for correctness.

If we wanted to use trait constraints with our `Convert` trait from the previous
section we could do so like so:

```sway
{{#include ../../../../examples/traits/src/main.sw:trait_constraint}}
```

This function allows you to take any generic data type `T` and convert it to the
type `Rectangle` _as long as `Convert<T>` is implemented for `Rectangle`_.
Calling this function with a type `T` for which `Convert<T>` is not implemented
for `Rectangle` will fail Sway's compile-time checks.
