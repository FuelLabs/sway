# Type Annotations

When declaring a variable it is possible to annotate it with a type; however, the compiler can usually infer that information automatically.

The general approach is to omit a type if the compiler does not throw an error; however, if it is deemed clearer by the developer to indicate the type then that is also encouraged.

```sway
{{#include ../../../code/language/style-guide/annotation_style/src/lib.sw:type_annotation}}
```
