# Inline

When making a call the compiler may generate code to call a function where it is defined or it may copy the function code (inline) to prevent additional code generation.

The Sway compiler automatically inlines functions based on internal heuristics; however, the `inline` attribute may be used to suggest, but not require, code generation or code copying.

## Generate code

To suggest code generation use the `never` keyword.

```sway
{{#include ../../../../code/language/annotations/src/main.sw:never_inline}}
```

## Copy code

To suggest code copy use the `always` keyword.

```sway
{{#include ../../../../code/language/annotations/src/main.sw:always_inline}}
```
