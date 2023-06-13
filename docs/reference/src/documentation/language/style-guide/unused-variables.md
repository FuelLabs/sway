# Unused Variables

A good practice is naming variables appropriately; however, variables may be unused at times such as the `timestamp` from the `call()`.

```sway
{{#include ../../../code/language/style-guide/unused_variables/src/lib.sw:unused_variable}}
```

## Named

We may preserve the name to provide context to the reader by prepending the variable with `_`.

```sway
{{#include ../../../code/language/style-guide/unused_variables/src/lib.sw:named_unused_variable}}
```

## Nameless

We may discard the context and the value by assigning it to `_`.

```sway
{{#include ../../../code/language/style-guide/unused_variables/src/lib.sw:nameless_variable}}
```
