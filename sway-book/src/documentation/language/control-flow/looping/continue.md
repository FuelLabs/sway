# continue

`continue` is a keyword available for use inside of a `while` loop and it allows us to skip to the next iteration without executing the code after it.

In this example the `while` loop will iterate until the `counter` is greater than or equal to `num`. During iteration if the value of `counter` is even then it will skip the summation and jump to the next iteration effectively adding together odd numbers.

```sway
{{#include ../../../../code/language/control_flow/src/lib.sw:continue_example}}
```
