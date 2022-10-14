# while

Loops in Sway are currently limited to `while` loops. This is what they look like:

```sway
{{#include ../../../../code/language/control_flow/src/lib.sw:single_loop}}
```

You need the `while` keyword, some condition (`value < 10` in this case) which will be evaluated each iteration, and a block of code inside the curly braces (`{...}`) to execute each iteration.

### Nested loops

You can also nest `while` loops if needed:

```sway
{{#include ../../../../code/language/control_flow/src/lib.sw:nested_loop}}
```
