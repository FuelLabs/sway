# Enums

_Enumerations_, or _enums_, are also known as _sum types_. An enum is a type that could be one of several variants. To declare an enum, you enumerate all potential variants.

Here, we have defined five potential colors. Each enum variant is just the color name. As there is no extra data associated with each variant, we say that each variant is of type `()`, or unit. The unit type can be replaced with other types.

```sway
{{#include ../../../code/language/built-ins/enums/src/basic_enum.sw:content}}
```

### Enums of Structs

It is also possible to have an enum variant contain extra data. Take a look at this more substantial example, which combines struct declarations with enum variants:

```sway
{{#include ../../../code/language/built-ins/enums/src/enum_of_structs.sw:content}}
```

### Enums of Enums

It is possible to define enums of enums:

```sway
{{#include ../../../code/language/built-ins/enums/src/enum_of_structs.sw:content}}
```

### Enum Memory Layout

Enums do have some memory overhead. To know which variant is being represented, Sway stores a one-word (8-byte) tag for the enum variant. The space reserved after the tag is equivalent to the size of the _largest_ enum variant. So, to calculate the size of an enum in memory, add 8 bytes to the size of the largest variant. For example, in the case of `Color` above, where the variants are all `()`, the size would be 8 bytes since the size of the largest variant is 0 bytes.
