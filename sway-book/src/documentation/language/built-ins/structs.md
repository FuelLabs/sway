# Structs

A struct in Sway is a `product type` which is a data structure that allows grouping of various types under a name that can be referenced, unlike a [tuple](tuples.md). The types contained in the struct are named and thus they can be referenced by their names as well.

## Declaration

The following syntax demonstrates the definition of a struct named `Foo` containing two fields - `bar`, a `u64`, and `baz`, a `bool`.

```sway
{{#include ../../../code/language/built-ins/structs/src/lib.sw:definition}}
```

## Instantiation

To instatiate a struct the name of the struct must be used followed by `{}` where the fields from the [declaration](#declaration) must be specified inside the brackets.

```sway
{{#include ../../../code/language/built-ins/structs/src/lib.sw:instantiation}}
```

## Destructuring

The fields of a struct can be accessed through destructuring.

```sway
{{#include ../../../code/language/built-ins/structs/src/lib.sw:destructuring}}
```
