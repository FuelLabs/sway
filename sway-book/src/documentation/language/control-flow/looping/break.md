# break

`break` is a keyword available for use inside of a `while` loop and it is used to exit out of the loop before the looping condition is met.

```sway
{{#include ../../../../code/language/control_flow/src/lib.sw:break_example}}
```

In the example above the `while` loop is set to iterate until `counter` reaches the value of `10` however the [if expression](../if-expressions.md) will break out of the loop once `counter` reaches the value of `6`.
