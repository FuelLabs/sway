# const

A `const` is similar to an [immutable let](./let.md#immutable) however there are a few differences.

- The constant is evaluated at compile-time
- A constant can be declared in any scope i.e. inside of a [function](../functions/index.md) and outside
- The `mut` keyword cannot be used

## Declaration

To define a constant the `const` keyword is used followed by a name and an assignment of a value.

```sway
{{#include ../../../code/language/variables/src/lib.sw:constants}}
```

The example above hardcodes the value of `5` however function calls may also be used alongside [built-in types](../built-ins/index.md).
