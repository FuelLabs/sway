# Structs

A struct in Sway is a `product type` which is a data structure that allows grouping of various types under a name that can be referenced, unlike a tuple. The types contained in the struct are named and thus they can be referenced by their names as well.

The following syntax demonstrates the definition of a struct named `Foo` containing two fields - `bar`, a `u64`, and `baz`, a `bool`.

```sway
{{#include ../../../code/language/built-ins/structs/src/lib.sw:definition}}
```

To instatiate a struct the name of the struct must be used followed by `{}` where the fields from the definition above must be specified inside the brackets.

```sway
{{#include ../../../code/language/built-ins/structs/src/lib.sw:instantiation}}
```

It's also possible to take a struct and access its fields through destructuring.

```sway
{{#include ../../../code/language/built-ins/structs/src/lib.sw:destructuring}}
```

### Struct Memory Layout

Structs have zero memory overhead meaning that each field is laid out sequentially in memory. No metadata regarding the struct's name or other properties is preserved at runtime. 

In other words, structs are compile-time constructs similar to Rust, but different in other languages with runtimes like Java.
