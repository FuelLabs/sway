# Enums

An enum, also known as a `sum type`, is a type that consists of several variants where each variant is named and has a type.

Let's take a look at an example where we define an enum called `Color` with a few color variations.

```sway
{{#include ../../../code/language/built-ins/enums/src/basic_enum.sw:definition}}
```

We begin by using the `enum` keyword followed by the name for our enumeration. The variants are contained inside `{}` and they are sequentially ordered from top to bottom. Each variant has a name, such as the first `Blue` variant, and a type, which in this case they are all the unit type `()`. 

The unit type is a type that does not contain any data however any type can be used.

```sway
{{#include ../../../code/language/built-ins/enums/src/basic_enum.sw:init}}
```

### Enums of Structs

In order to demonstrate more complex data types we can define a struct and assign that struct as a data type for any of an enum's variants.

Here we have a struct `Item` and an enum `MyEnum`. The enum has one variant by the name `Product` and its type is declared to the right of `:` which in this case is our struct `Item`.

```sway
{{#include ../../../code/language/built-ins/enums/src/enum_of_structs.sw:content}}
```

### Enums of Enums

Similar to structs we can use other enums as types for our variants.

```sway
{{#include ../../../code/language/built-ins/enums/src/enum_of_enums.sw:content}}
```

### Enum Memory Layout

Enums do have some memory overhead. To know which variant is being represented, Sway stores a one-word (8-byte) tag for the enum variant. The space reserved after the tag is equivalent to the size of the _largest_ enum variant. So, to calculate the size of an enum in memory, add 8 bytes to the size of the largest variant. For example, in the case of `Color` above, where the variants are all `()`, the size would be 8 bytes since the size of the largest variant is 0 bytes.
