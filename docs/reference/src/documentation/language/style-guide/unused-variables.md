# Unused Variables

A good practice is naming variables appropriately; however, variables may be unused at times such as the `timestamp` from the `call()`.

```sway
{{#include ../../../code/language/style-guide/unused_variables/src/lib.sw:named_unused_variable}}
```

When a variable is unused we can discard its value by assigning it to `_`.

```sway
{{#include ../../../code/language/style-guide/unused_variables/src/lib.sw:unnamed_unused_variable}}
```
