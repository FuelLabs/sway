# Constants

Constants are similar to [immutable let](./let.md#immutable) variables; however, there are a few differences:

- Constants are always evaluated at compile-time
- Constants can be declared both inside of a [function](../functions/index.md) and at global scope.
- The `mut` keyword cannot be used with constants.

## Declaration

To define a constant the `const` keyword is used followed by a name and an assignment of a value.

```sway
{{#include ../../../code/language/variables/src/lib.sw:constants}}
```

The example above hardcodes the value of `5` however function calls may also be used alongside [built-in types](../built-ins/index.md).
