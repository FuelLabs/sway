# Strings

Sway strings are declared using double-quotes `"`. Single quotes `'` cannot be used. Attempting to define a string with single-quotes will result in an error.

```sway
{{#include ../../../code/misc/known-issues/strings/src/lib.sw:single_quotes}}
```

Strings are UTF-8 encoded therefore they cannot be indexed.

```sway
{{#include ../../../code/misc/known-issues/strings/src/lib.sw:indexing}}
```
