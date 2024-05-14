# Structs

A struct in Sway is a `product type` which is a data structure that allows grouping of various types under a name that can be referenced, unlike a [tuple](tuples.md). The types contained in the struct are named and thus they can be referenced by their names as well.

## Declaration

The following syntax demonstrates the declaration of a struct named `Foo` containing two fields - public field `bar`, a `u64`, and a private field `baz`, a `bool`.

```sway
{{#include ../../../code/language/built-ins/structs/src/lib.sw:definition}}
```

Public fields are accessible in all the modules in which the struct is accessible. Private fields are accessible only within the module in which the struct is declared.

## Instantiation

To instantiate a struct the name of the struct must be used followed by `{}` where the fields from the [declaration](#declaration) must be specified inside the brackets. Instantiation requires all fields to be initialized, both private and public.

```sway
{{#include ../../../code/language/built-ins/structs/src/lib.sw:instantiation}}
```

Structs with private fields can be instantiated only within the module in which the struct is declared.

## Destructuring

The fields of a struct can be accessed through destructuring.

```sway
{{#include ../../../code/language/built-ins/structs/src/lib.sw:destructuring}}
```

When destructuring structs with private fields outside of a module in which the struct is defined, the private fields must be omitted by using the `..`.
