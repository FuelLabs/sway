# Boolean Type

A boolean is a type that is represented by either a value of one or a value of zero. To make it easier to use the values have been given names and they are `true` and `false`.

Boolean values are typically used for conditional logic or validation, for example in [if expressions](../control-flow/if-expressions.md), and thus expressions are said to be evaluated to `true` or `false`.

Something that can be done with a boolean type is to flip its value from `true` to `false` or `false` to `true` using the unary negation operator `!`. 

In the example below we create two boolean variables and [implicitly](../functions/return.md) return a comparison of the values. A `!` is used on the `is_false` variable which will flip its value from `false` to `true` and thus the comparison equates to `true == true`, which is true, and thus the returned boolean value will be `true`.

```sway
{{#include ../../../code/language/built-ins/booleans/src/lib.sw:syntax}}
```
