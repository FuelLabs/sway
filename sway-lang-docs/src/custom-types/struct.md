# Structs

Structs in Sway are a named grouping of types. You may also be familiar with structs via another name: _product types_. Sway does not make any significantly unique usages of structs; they are similar to most other languages which have structs. If you're coming from an object-oriented background, a struct is like the data attributes of an object.

Firstly, we declare a struct named `Foo` with two fields. The first field is named `bar` and it accepts values of type `u64`, the second field is named `baz` and it accepts `bool` values.

```sway
{{#include ../../../examples/structs/src/data_structures.sw}}
```

In order to instantiate the struct we use _struct instantiation syntax_, which is very similar to the declaration syntax except with expressions in place of types.

There are three ways to instantiate the struct.

- Hardcoding values for the fields
- Passing in variables with names different than the struct fields
- Using a shorthand notation via variables that are the same as the field names

```sway
{{#include ../../../examples/structs/src/main.sw}}
```

> **Note**
> You can mix and match all 3 ways to instantiate the struct at the same time.
> Moreover, the order of the fields does not matter when instantiating however we encourage declaring the fields in alphabetical order and instantiating them in the same alphabetical order

Furthermore, multiple variables can be extracted from a struct using the destructuring syntax.

### Struct Memory Layout

> **Note**
> This information is not vital if you are new to the language, or programming in general

Structs have zero memory overhead. What that means is that in memory, each struct field is laid out sequentially. No metadata regarding the struct's name or other properties is preserved at runtime. In other words, structs are compile-time constructs. This is the same in Rust, but different in other languages with runtimes like Java.
