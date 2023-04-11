# Constants

Constants are similar to [immutable let](./let.md#immutable) variables; however, there are a few differences:

- Constants are always evaluated at compile-time
- Constants can be declared both inside of a [function](../functions/index.md) and at global/`impl` scope.
- The `mut` keyword cannot be used with constants.

## Declaration

To define a constant the `const` keyword is used followed by a name and an assignment of a value.

```sway
{{#include ../../../code/language/variables/src/lib.sw:constants}}
```

The example above hardcodes the value of `5` however function calls may also be used alongside [built-in types](../built-ins/index.md).

## `impl` consts

Constants can also be declared inside `impl` blocks. In this case, the constant is referred to as an associated const.

```sway
struct Point {
    x: u64,
    y: u64,
}

impl Point {
    const ZERO: Point = Point { x: 0, y: 0 };
}

fn main() -> u64  {
    Point::ZERO.x
}
```
