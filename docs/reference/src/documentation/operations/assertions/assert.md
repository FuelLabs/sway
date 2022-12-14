# assert

The `assert` function is automatically imported into every program from the [prelude](../../misc/prelude.md) and it takes an expression which must evaluate to a [Boolean](../../language/built-ins/boolean.md). If the Boolean is `true` then nothing will happen and the code will continue to run otherwise the virtual machine will revert.

## Example

Here we have a function which takes two `u64` arguments and subtracts them. A `u64` cannot be negative therefore the assertion enforces that `b` must be less than or equal to `a`.

If the condition is not met, then the virtual machine will revert.

```sway
{{#include ../../../code/operations/assertions/src/lib.sw:assert}}
```
