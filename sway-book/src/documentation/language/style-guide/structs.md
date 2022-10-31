# Struct Shorthand

A [struct](../built-ins/structs.md) has a shorthand notation for initializing its fields. The shorthand works by passing a variable into a struct with the exact same name and type.

The following struct has a field `amount` with type `u64`.

```sway
{{#include ../../../code/language/style-guide/struct_shorthand/src/lib.sw:struct_shorthand_definition}}
```

Using the shorthand notation we can initialize the struct in the following way.

```sway
{{#include ../../../code/language/style-guide/struct_shorthand/src/lib.sw:struct_shorthand_use}}
```

The shorthand is encouraged because it is a cleaner alternative to the following.

```sway
{{#include ../../../code/language/style-guide/struct_shorthand/src/lib.sw:struct_shorthand_avoid}}
```

