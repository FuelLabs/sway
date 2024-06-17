
# Namespace

Namespaces can be used on a `storage` block and variables placed inside the namespaces. A single `storage` block may contain multiple namespaces placed sequentially and or nested.

The hash calculations determining the position of variables in a block with namespace `my_namespace` that contains the variable `foobar` are calculated from `sha256("storage::my_namespace.foobar")`.

## Example

A namespace can be declared as follows:

```sway
{{#include ../../../../code/language/annotations/src/main.sw:storage_namespace}}
```

A variable inside a namespace can be accessed as follows:

```sway
{{#include ../../../../code/language/annotations/src/main.sw:storage_namespace_access}}
```
