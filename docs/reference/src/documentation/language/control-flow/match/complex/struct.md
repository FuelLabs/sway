# Structs

We can match on specific arguments inside a struct while ignoring the rest by using `..`.

```sway
{{#include ../../../../../code/language/control_flow/src/lib.sw:complex_struct_unpacking_match}}
```

If the struct is imported from another module and has private fields, the private fields must always be ignored by using `..`.
