
# Namespace

A namespace attribute on a `storage` block will position the slots of the variables in that `storage` block uniquely based on the name in the attribute's argument.

The hash calculations determining the position of variables in a block annotated with `#[namespace(foobar)]` will contain `foobar`.

## Example

```sway
{{#include ../../../../code/language/annotations/src/main.sw:storage_namespace}}
```
