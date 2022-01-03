# Libraries

Libraries in Sway are files used to define out new types and functions useful for general-purpose programming. An example of this is the [Sway Core Library](https://github.com/FuelLabs/sway-lib-core) which outlines various functions that attach to the uint64 type. 

Functions in Libraries can also read from storage and interact with the state. Libraries are denoted using the `library` keyword followed by a name so that they can be imported.

```sway
library my_library;
```

A good reference library to use when learning about designing your own is [Sway Core Library](https://github.com/FuelLabs/sway-lib-core). The `add` function that is exported is done by creating an `Add` trait and implementing it for `u64`, attaching this library function to the type, so that when imported, `u64`s can utilize the `add` function.

```sway
pub trait Add {
    fn add(self, other: Self) -> Self;
}

impl Add for u64 {
    fn add(self, other: Self) -> Self {
        asm(r1: self, r2: other, r3) {
            add r3 r2 r1;
            r3: u64
        }
    }
}
```

which defines a trait, then implements it for the `u64` type with an implementation. This gives all `u64`s the `add` function, which is inserted at compile time when you use the `+` operator in Sway. Libraries don't just have to export functions attached to types though, you can also use libraries to just export types like below.

```sway
pub struct MyStruct {
    field_one: u64,
    field_two: bool,
}
```

Then contracts which import these libraries will be able to access this type.

Libraries can be imported using the `use` keyword and with a `::` separating the name of the library and the import.

Here is an example of importing storage and its related functions from the standard library. 

```sway
use std::storage::*;
```

Wildcard imports using `*` are supported, but it is always recommended to use explicit imports where possible. Note that multiple imports are not yet supported: https://github.com/FuelLabs/sway/issues/563.
