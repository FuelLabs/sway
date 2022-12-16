# Libraries

Libraries in Sway are files used to define new common behavior. The most prominent example of this is the [Sway Standard Library](../introduction/standard_library.html).

## Writing Libraries

Libraries are defined using the `library` keyword at the beginning of a file, followed by a name so that they can be imported.

```sway
library my_library;

// library code
```

A good reference library to use when learning library design is the [Sway Standard Library](../introduction/standard_library.html). For example, the standard library offers an [implementation](https://github.com/FuelLabs/sway/blob/master/sway-lib-std/src/option.sw) of `enum Option<T>` which is a generic type that represents either the existence of a value using the variant `Some(..)` or a value's absence using the variant `None`. The [Sway file implementing `Option<T>`](https://github.com/FuelLabs/sway/blob/master/sway-lib-std/src/option.sw) has the following structure:

* The `library` keyword followed by the name of the library:

```sway
library option;
```

* A `use` statement that imports `revert` from another library _inside_ the standard library:

```sway
use ::revert::revert;
```

* The `enum` definition which starts with the keyword `pub` to indicate that this `Option<T>` is publically available _outside_ the `option` library:

```sway
pub enum Option<T> {
    // variants
}
```

* An `impl` block that implements some methods for `Option<T>`:

```sway
impl<T> Option<T> {

    fn is_some(self) -> bool {
        // body of is_some
    }

    // other methods
}
```

Now that the library `option` is fully written, and because `Option<T>` is defined with the `pub` keyword, we are now able to import `Option<T>` using `use std::option::Option;` from any Sway project and have access to all of its variants and methods. That being said, `Option` is automatically available in the [standard library prelude](../introduction/standard_library.md#standard-library-prelude) so you never actually have to import it manually.

Libraries are composed of just a `Forc.toml` file and a `src` directory, unlike contracts which usually contain a `tests` directory and a `Cargo.toml` file as well. An example of a library's `Forc.toml`:

```toml
[project]
authors = ["Fuel Labs <contact@fuel.sh>"]
entry = "lib.sw"
license = "Apache-2.0"
name = "my_library"

[dependencies]
```

which denotes the authors, an entry file, the name by which it can be imported, and any dependencies.

For large libraries, it is recommended to have a `lib.sw` entry point re-export all other sub-libraries. For example, the `lib.sw` of the standard library looks like:

```sway
library std;

dep block;
dep storage;
dep constants;
// .. Other deps
```

with other libraries contained in the `src` folder, like the block library (inside of `block.sw`):

```sway
library block;

// Implementation of the `block` library 
```

The `dep` keyword in the main library includes a dependency on another library, making all of its items (such as functions and structs) accessible from the main library. The `dep` keyword simply makes the library a dependency and fully accessible within the current context.

## Using Libraries

Libraries can be imported using the `use` keyword and with a `::` separating the name of the library and the import.

Here is an example of importing the `get<T>` and `store<T>` functions from the `storage` library.

```sway
use std::storage::{get, store};
```

Wildcard imports using `*` are supported, but it is always recommended to use explicit imports where possible.

Libraries _other than the standard library_ have to be added as a dependency in `Forc.toml`. This can be done by adding a path to the library in the `[dependencies]` section. For example:

```toml
wallet_lib = { path = "/path/to/wallet_lib" }
```

> **Note**: the standard library is implicitly available to all Forc projects, that is, you are not required to manually specify `std` as an explicit dependency in `Forc.toml`.
