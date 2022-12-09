# revert

The `revert` function is automatically imported into every program from the [prelude](../../misc/prelude.md) and it takes a `u64` as an exit code.

The function will behave differently depending on the context in which it is used:

- When used inside a [predicate](../../language/program-types/predicate.md) the function will panic and crash the program
- Otherwise it will revert the virutal machine

## Example

To manually force a revert we need to provide an exit code. To be able to distinguish one revert from another different exit codes can be used in different places.

```sway
{{#include ../../../code/operations/assertions/src/lib.sw:revert}}
```
