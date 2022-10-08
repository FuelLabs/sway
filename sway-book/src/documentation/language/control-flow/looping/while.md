# while

Loops in Sway are currently limited to `while` loops. This is what they look like:

```sway
{{#include ../../../../code/language/control_flow/src/lib.sw:single_loop}}
```

You need the `while` keyword, some condition (`value < 10` in this case) which will be evaluated each iteration, and a block of code inside the curly braces (`{...}`) to execute each iteration.

### `break` and `continue`

`break` and `continue` keywords are available to use inside the body of a `while` loop. The purpose of the `break` statement is to break out of a loop early:

```sway
{{#include ../../../../code/language/control_flow/src/lib.sw:break_example}}
```

The purpose of the `continue` statement is to skip a portion of a loop in an iteration and jump directly into the next iteration:

```sway
{{#include ../../../../code/language/control_flow/src/lib.sw:continue_example}}
```

### Nested loops

You can also use nested `while` loops if needed:

```sway
{{#include ../../../../code/language/control_flow/src/lib.sw:nested_loop}}
```
