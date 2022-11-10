# Structs, Tuples, and Enums

## Structs

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

## Tuples

Tuples are a [basic static-length type](./built_in_types.md#tuple-types) which contain multiple different types within themselves. The type of a tuple is defined by the types of the values within it, and a tuple can contain basic types as well as structs and enums.

You can access values directly by using the `.` syntax. Moreover, multiple variables can be extracted from a tuple using the destructuring syntax.

```sway
{{#include ../../../examples/tuples/src/main.sw}}
```

## Enums

_Enumerations_, or _enums_, are also known as _sum types_. An enum is a type that could be one of several variants. To declare an enum, you enumerate all potential variants.

Here, we have defined five potential colors. Each enum variant is just the color name. As there is no extra data associated with each variant, we say that each variant is of type `()`, or unit.

```sway
{{#include ../../../examples/enums/src/basic_enum.sw}}
```

### Enums of Structs

It is also possible to have an enum variant contain extra data. Take a look at this more substantial example, which combines struct declarations with enum variants:

```sway
{{#include ../../../examples/enums/src/enum_of_structs.sw}}
```

### Enums of Enums

It is possible to define enums of enums:

```sway
{{#include ../../../examples/enums/src/enum_of_enums.sw}}
```

#### Preferred usage

The preferred way to use enums is to use the individual (not nested) enums directly because they are easy to follow and the lines are short:

```sway
{{#include ../../../examples/enums/src/enums_preferred.sw}}
```

#### Inadvisable

If you wish to use the nested form of enums via the `Error` enum from the example above, then you can instantiate them into variables using the following syntax:

```sway
{{#include ../../../examples/enums/src/enums_avoid.sw}}
```

Key points to note:

- You must import all of the enums you need instead of just the `Error` enum
- The lines may get unnecessarily long (depending on the names)
- The syntax is not the most ergonomic

### Enum Memory Layout

> **Note**
> This information is not vital if you are new to the language, or programming in general.

Enums do have some memory overhead. To know which variant is being represented, Sway stores a one-word (8-byte) tag for the enum variant. The space reserved after the tag is equivalent to the size of the _largest_ enum variant. So, to calculate the size of an enum in memory, add 8 bytes to the size of the largest variant. For example, in the case of `Color` above, where the variants are all `()`, the size would be 8 bytes since the size of the largest variant is 0 bytes.
