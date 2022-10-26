# require

The `require` function is automatically imported into every program from the [prelude](../../misc/prelude.md) and it takes an expression which must evaluate to a boolean. If the boolean is `true` then nothing will happen and the rest of the code will continue to run otherwise a log will be emitted and the virtual machine will revert.

## Example

Here we have a function which takes two `u64` arguments and subtracts them. A `u64` cannot be negative therefore the assertion enforces that `b` must be less than or equal to `a`. 

If the condition is not met then the message `b is too large` will be logged and the virtual machine will revert.

The message is generic therefore it can be any type, in this example it's a string.

```sway
{{#include ../../../code/operations/assertions/src/req.sw:require}}
```
