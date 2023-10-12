# Deprecated

This annotation marks an item as deprecated, which makes the compiler to emit a warning for each usage of the item. This warning can be disabled using `#[allow(deprecated)]`.

It is also possible to customize the warning message using the argument `note`.

```sway
{{#include ../../../../code/language/annotations/src/main.sw:allow_deprecated_annotation}}
```
