# assert_eq

The `assert_eq` function is automatically imported into every program from the [prelude](../../misc/prelude.md). It takes two expressions which are compared and the result is a [Boolean](../../language/built-ins/boolean.md). If the value is `false` then the virtual machine will revert.

## Example

Here is a function which asserts that `a` & `b` must be equal.

```sway
{{#include ../../../code/operations/assertions/src/lib.sw:assert_eq}}
```
