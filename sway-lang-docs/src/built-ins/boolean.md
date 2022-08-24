# Boolean Type

The boolean type (`bool`) has two potential values: `true` or `false`. 

Boolean values are typically used for conditional logic or validation, for example in [if expressions](../control-flow/if-else-if-else.md). Booleans can be negated, or flipped, with the unary negation operator `!`. 

For example:

```sway
fn returns_false() -> bool {
    let boolean_value = true;
    !boolean_value
}
```
