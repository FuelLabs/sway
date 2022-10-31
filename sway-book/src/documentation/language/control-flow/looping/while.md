# while

A `while` loop uses the `while` keyword followed by a condition which evaluates to a [boolean](../../built-ins/boolean.md).

```sway
{{#include ../../../../code/language/control_flow/src/lib.sw:single_loop}}
```

In the example above we use two conditions.

1. If the `counter` is less than `10` then continue to iterate
2. If the `condition` variable is `true` then continue to iterate

As long as both those conditions are `true` then the loop will iterate. In this case the loop will finish iterating once `counter` reaches the value of `6` because `condition` will be set to `false`.

### Nested loops

Sway also allows nested `while` loops.

```sway
{{#include ../../../../code/language/control_flow/src/lib.sw:nested_loop}}
```
