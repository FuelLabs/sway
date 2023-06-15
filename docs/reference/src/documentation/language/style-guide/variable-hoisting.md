# Variable Hoisting

Hoisting refers to moving variables to the top of the function scope.

## Preferred

The advantage of hoisting is a single place to find the declarations of variables.

```sway
{{#include ../../../code/language/style-guide/variable_hoisting/src/lib.sw:hoisting_variables}}
```

## Alternative

Variable declarations may be beside the code that uses them; however, this forces the reader to remember the code or re-read the function.

```sway
{{#include ../../../code/language/style-guide/variable_hoisting/src/lib.sw:grouping_variables}}
```
