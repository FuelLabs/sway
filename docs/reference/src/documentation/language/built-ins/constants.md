# Constants

A constant in Sway is similar to a variable; however, there are a few differences:

- Constants are always evaluated at compile-time
- Constants can be declared both inside of a [function](../index.md) and at global / `impl` scope.
- The `mut` keyword cannot be used with constants.

## Associated Constants

Associated constants are constants associated with a type and can be declared in an `impl` block or in a `trait` definition.

Associated constants declared inside a `trait` definition may omit their initializers to indicate that each implementation of the trait must specify those initializers.

The identifier is the name of the constant used in the path. The type is the type that the
definition has to implement.

You can _define_ an associated const directly in the interface surface of a trait:

```sway
script;

trait ConstantId {
    const ID: u32 = 0;
}
```

Alternatively, you can also _declare_ it in the trait, and implement it in the interface of the
types implementing the trait.

```sway
script;

trait ConstantId {
    const ID: u32;
}

struct Struct {}

impl ConstantId for Struct {
    const ID: u32 = 1;
}

fn main() -> u32 {
    Struct::ID
}
```

### `impl self` consts

Constants can also be declared inside non-trait `impl` blocks.

```sway
script;

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

The following syntax also demonstrates the definition of a struct named `S` containing a constant `ID`.

```sway
{{#include ../../../code/language/built-ins/constants/src/lib.sw}}
```
