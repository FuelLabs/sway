# Boolean Type

A boolean is a type that is represented by either a value of one or a value of zero. To make it easier to use the values have been given names: `true` & `false`.

Boolean values are typically used for conditional logic or validation, for example in [if expressions](../control-flow/if-expressions.md), and thus expressions are said to be evaluated to `true` or `false`.

Using the unary operator `!` the boolean value can be changed:

- From `true` to `false`
- From `false` to `true`

## Example

The following example creates two boolean [variables](../variables/index.md), performs a comparison using the unary operator and [implicitly](../functions/return.md) returns the result.

```sway
{{#include ../../../code/language/built-ins/booleans/src/lib.sw:syntax}}
```
