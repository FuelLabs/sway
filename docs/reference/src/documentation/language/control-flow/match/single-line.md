# Single Line Arm

The following example demonstrates how a type can be matched on and its output is assigned to a variable. The assignment to a variable is optional.

```sway
{{#include ../../../../code/language/control_flow/src/lib.sw:simple_match}}
```

The left side of the arrow `=>` is the pattern that we are matching on and the right side of the arrow `=>` is the logic that we want to perform, in this case we are returning a different multiple of `10` depending on which arm is matched.

We check each arm starting from `0` and make our way down until we either find a match on our pattern or we reach the `catch_all` case.

The `catch_all` case is equivalent to an `else` in [if expressions](../if-expressions.md) and it does not have to be called `catch_all`. Any pattern declared after a `catch_all` case will not be matched because once the compiler sees the first `catch_all` it stop performing further checks.
