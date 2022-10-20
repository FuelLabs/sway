# Tuples

A tuple is a general-purpose static-length aggregation of types, in other words, it's a single type that consists of an aggregate of zero or more types. The internal types that make up a tuple, and the tuple's arity, define the tuple's type.

## Usage

To declare a tuple we wrap the values in `()`.

```sway
{{#include ../../../code/language/built-ins/tuples/src/lib.sw:declare}}
```

Values can be retrieved individually from the tuple by specifying the index.

```sway
{{#include ../../../code/language/built-ins/tuples/src/lib.sw:index}}
```

A value can be mutated in a tuple as long as the tuple is declared to be [mutable](../variables/index.md) and the new value has the same type as the previous value.

```sway
{{#include ../../../code/language/built-ins/tuples/src/lib.sw:internal_mutability}}
```

The entire tuple can be overwritten when it is [mutable](../variables/index.md) and the type for each value is the same.

```sway
{{#include ../../../code/language/built-ins/tuples/src/lib.sw:mutability}}
```

Elements can be desctructured from a tuple into individual variables.

```sway
{{#include ../../../code/language/built-ins/tuples/src/lib.sw:destructure}}
```

We can also ignore elements when destructoring.

```sway
{{#include ../../../code/language/built-ins/tuples/src/lib.sw:ignore_destructure}}
```
