# Enums

> TODO: intro, show enums

## Preferred usage

The preferred way to use enums is to use the individual (not nested) enums directly because they are easy to follow and the lines are short:

```sway
{{#include ../../../code/language/style-guide/enums/src/lib.sw:use}}
```

## Inadvisable

If you wish to use the nested form of enums via the `Error` enum from the example above, then you can instantiate them into variables using the following syntax:

```sway
{{#include ../../../code/language/style-guide/enums/src/lib.sw:avoid}}
```

Key points to note:

- You must import all of the enums you need instead of just the `Error` enum
- The lines may get unnecessarily long (depending on the names)
- The syntax is not the most ergonomic
