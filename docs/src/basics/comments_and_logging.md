# Comments and Logging

## Comments

Comments in Sway start with two slashes and continue until the end of the line. For comments that extend beyond a single line, you'll need to include `//` on each line.

```sway
// hello world
```

```sway
// let's make a couple of lines
// commented.
```

You can also place comments at the ends of lines containing code.

```sway
fn main() {
    let baz = 8; // Eight is a lucky number
}
```

## Logging

To log integers, you can use the `log_u64`, `log_u32`, `log_u16`, or `log_u8` functions from the standard library.

```sway
use std::chain::log_u64;

fn main() {
    let baz = 8;
    log_u64(baz);
}
```

Note that you cannot log arbitrary structs yet because [we do not yet support serialization](../reference/temporary_workarounds.html#serialization-and-deserialization).
