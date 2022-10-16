# continue

`continue` is a keyword available for use inside of a `while` loop and it is used to skip to the next iteration without executing the code after `continue`.

```sway
{{#include ../../../../code/language/control_flow/src/lib.sw:continue_example}}
```

In the example above the `while` loop is set to iterate until `counter` reaches the value of `10` however the [if expression](../if-expressions.md) will skip (not execute) the "other code" when `counter` is an even value. For example, this could be used to add all the odd numbers from `0` to `10`.
