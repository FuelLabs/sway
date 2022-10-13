# Multiple Values

We can `match` on multiple values by wrapping them in a tuple and then specifying each variant in the same structure (tuple) that they have been defined.

```sway
{{#include ../../../../../code/language/control_flow/src/lib.sw:complex_multi_arg_enum_match}}
```
