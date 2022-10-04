# Getter Functions

Functions that return values typically follow one of two styles:

- Prepending `get_` to the start of the name
- Omitting `get_`

In Sway the encouraged usage is to omit the `get_` prefix.

```sway
{{#include ../../../code/language/style-guide/getters/src/lib.sw:use}}
```

That is to say the following is discouraged.

```sway
{{#include ../../../code/language/style-guide/getters/src/lib.sw:avoid}}
```
