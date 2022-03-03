# Libraries

Libraries in Sway are files used to define new common behavior. An example of this is the [Sway Core Library](https://github.com/FuelLabs/sway-lib-core) which outlines various methods that the `u64` type implements.

## Writing Libraries

Functions in libraries can also read from storage and interact with the state. Libraries are denoted using the `library` keyword at the beginning of the file, followed by a name so that they can be imported. E.g. `library foo;`.

```sway
library my_library;
```

A good reference library to use when learning library design is the [Sway Core Library](https://github.com/FuelLabs/sway-lib-core). The `add` function interface is defined via the `Add` trait and then implemented for `u64`. This attaches this `add` function to the type so that, when the trait is imported, `u64`s can utilize the `add` function.

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

This snippet defines the trait `Add`, then implements it for the `u64` type by providing a function body. This gives all `u64`s the `add` function, which is inserted at compile time when you use the `+` operator in Sway. Libraries can export more than functions, though. You can also use libraries to just export types like below.

```sway
pub struct MyStruct {
    field_one: u64,
    field_two: bool,
}
```

Libraries are composed of just a `Forc.toml` file and a `src` folder, unlike usual Sway projects which usually contain a `tests` folder and a `Cargo.toml` file as well. An example of a Library's `Forc.toml`: 

```toml=
[project]
authors = ["Fuel Labs <contact@fuel.sh>"]
entry = "lib.sw"
license = "Apache-2.0"
name = "lib-std"

[dependencies]
"core" = { git = "http://github.com/FuelLabs/sway-lib-core" }
```

which denotes the author, an entry file, the name by which it can be imported, and any dependencies. For large libraries, it is recommended to have a `lib.sw` entry point re-export all other sub-libraries. For example, the `lib.sw` of the standard library looks like:

```sway
library std;

dep block;
dep storage;
dep constants;
```

with other libraries contained in the `src` folder, like the block library (inside of `block.sw`):

```sway
library block;
//! Functionality for accessing block-related data.

/// Get the current block height
pub fn height() -> u64 {
    asm(height) {
        bhei height;
        height: u64
    }
}
```

The `dep` keyword in the main library includes a dependency on another library, making all of its items (such as functions and structs) accessible from the main library. The `dep` keyword simply makes the library a dependency and fully accessible within the current context.

## Using Libraries

Libraries can be imported using the `use` keyword and with a `::` separating the name of the library and the import.

Here is an example of importing storage and its related functions from the standard library.

```sway
use std::storage::*;
```

Wildcard imports using `*` are supported, but it is always recommended to use explicit imports where possible.
