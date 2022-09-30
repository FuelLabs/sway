# Enums

_Enumerations_, or _enums_, are also known as _sum types_. An enum is a type that could be one of several variants. To declare an enum, you enumerate all potential variants.

Here, we have defined five potential colors. Each enum variant is just the color name. As there is no extra data associated with each variant, we say that each variant is of type `()`, or unit.

> **Note**
> enum instantiation does not require the `~` tilde syntax

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
