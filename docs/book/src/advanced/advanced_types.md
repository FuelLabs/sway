# Advanced Types

## Creating Type Synonyms with Type Aliases

Sway provides the ability to declare a type alias to give an existing type another name. For this we use the `type` keyword. For example, we can create the alias `Kilometers` to `u64` like so:

```sway
{{#include ../../../../examples/type_aliases/src/main.sw:type_alias}}
```

Now, the alias `Kilometers` is a _synonym_ for `u64`. Note that `Kilometers` is **not** a separate new type. Values that have the type `Kilometers` will be treated the same as values of type `u64`:

```sway
{{#include ../../../../examples/type_aliases/src/main.sw:addition}}
```

Because `Kilometers` and `u64` are the same type, we can add values of both types and we can pass `Kilometers` values to functions that take `u64` parameters. However, using this method, we don’t get the type checking benefits that we get from introducing a _separate_ new type called `Kilometers`. In other words, if we mix up `Kilometers` and `i32` values somewhere, the compiler will not give us an error.

The main use case for type synonyms is to reduce repetition. For example, we might have a lengthy array type like this:

```sway
[MyStruct<u64, b256>; 5]
```

Writing this lengthy type in function signatures and as type annotations all over the code can be tiresome and error prone. Imagine having a project full of code like this:

```sway
{{#include ../../../../examples/type_aliases/src/main.sw:long_type_use}}
```

A type alias makes this code more manageable by reducing the repetition. Below, we’ve introduced an alias named `MyArray` for the verbose type and can replace all uses of the type with the shorter alias `MyArray`:

```sway
{{#include ../../../../examples/type_aliases/src/main.sw:long_type_use_shorter}}
```

This code is much easier to read and write! Choosing a meaningful name for a type alias can help communicate your intent as well.
