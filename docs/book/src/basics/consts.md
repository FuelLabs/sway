# Constants

Constants are similar to variables; however, there are a few differences:

- Constants are always evaluated at compile-time
- Constants can be declared both inside of a [function](../index.md) and at global / `impl` scope.
- The `mut` keyword cannot be used with constants.

```sway
const ID: u32 = 0;
```

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

## Configurable Constants

Configurable constants are special constants that behave like regular constants in the sense that they cannot change during program execution, but they can be configured _after_ the Sway program has been built. The Rust and TS SDKs allow updating the values of these constants by injecting new values for them directly in the bytecode without having to build the program again. These are useful for contract factories and behave somewhat similarly to `immutable` variables from languages like Solidity.

Configurable constants are declared inside a `configurable` block and require a type ascription and an initializer as follows:

```sway
{{#include ../../../../examples/configurable_constants/src/main.sw:configurable_block}}
```

At most one `configurable` block is allowed in a Sway project. Moreover, `configurable` blocks are not allowed in libraries.

Configurable constants can be read directly just like regular constants:

```sway
{{#include ../../../../examples/configurable_constants/src/main.sw:using_configurables}}
```
