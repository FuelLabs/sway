# Enums

An [`enum`](../../language/built-ins/enums.md) may contain many types including other enums.

```sway
{{#include ../../../code/language/style-guide/enum_style/src/lib.sw:style_enums}}
```

## Encouraged

The preferred way to use [`enums`](../built-ins/enums.md) is to use the individual (not nested) enums directly because they are easy to follow and the lines are short:

```sway
{{#include ../../../code/language/style-guide/enum_style/src/lib.sw:use}}
```

## Discouraged

If you wish to use the nested form of enums via the `Error` enum from the example above, then you can instantiate them into variables using the following syntax:

```sway
{{#include ../../../code/language/style-guide/enum_style/src/lib.sw:avoid}}
```

Key points to note:

- You must import all of the enums i.e. `Error`, `StateError` & `UserError`
- The lines may get unnecessarily long (depending on the names)
- The syntax is unergonomic
