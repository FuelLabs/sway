# Libraries

<!-- This section should explain what a library is -->
<!-- library:example:start -->
Libraries in Sway are files used to define new common behavior.
<!-- library:example:end -->

The most prominent example of this is the [Sway Standard Library](../introduction/standard_library.md) that is made implicitly available to all Forc projects created using `forc new`.

## Writing Libraries

<!-- This section should explain how libraries are defined -->
<!-- def_lib:example:start -->
Libraries are defined using the `library` keyword at the beginning of a file, followed by a name so that they can be imported.
<!-- def_lib:example:end -->

```sway
library;

// library code
```

A good reference library to use when learning library design is the [Sway Standard Library](../introduction/standard_library.md). For example, the standard library offers an [implementation](https://github.com/FuelLabs/sway/blob/master/sway-lib-std/src/option.sw) of `enum Option<T>` which is a generic type that represents either the existence of a value using the variant `Some(..)` or a value's absence using the variant `None`. The [Sway file implementing `Option<T>`](https://github.com/FuelLabs/sway/blob/master/sway-lib-std/src/option.sw) has the following structure:

- The `library` keyword:

```sway
library;
```

- A `use` statement that imports `revert` from another library _inside_ the standard library:

```sway
use ::revert::revert;
```

- The `enum` definition which starts with the keyword `pub` to indicate that this `Option<T>` is publicly available _outside_ the `option` library:

```sway
pub enum Option<T> {
    // variants
}
```

- An `impl` block that implements some methods for `Option<T>`:

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

For large libraries, it is recommended to have a `lib.sw` entry point re-export all other sub-libraries.

<!-- This section should explain the `mod` keyword -->
<!-- mod:example:start -->
The `mod` keyword registers a submodule, making its items (such as functions and structs) accessible from the parent library.
If used at the top level it will refer to a file in the `src` folder and in other cases in a folder named after the library in which it is defined.
<!-- mod:example:end -->

For example, the `lib.sw` of the standard library looks like:

```sway
library;

mod block;
mod storage;
mod constants;
mod vm;
// .. Other deps
```

with other libraries contained in the `src` folder, like the `vm` library (inside of `src/vm.sw`):

```sway
library;

mod evm;
// ...
```

and it's own sub-library `evm` located in `src/vm/evm.sw`:

```sway
library;

// ...
```

## Using Libraries

There are two types of Sway libraries, based on their location and how they can be imported.

### Internal Libraries

Internal libraries are located within the project's `src` directory alongside
`main.sw` or in the appropriate folders as shown below:

```bash
$ tree
.
├── Cargo.toml
├── Forc.toml
└── src
    ├── internal_lib.sw
    ├── main.sw
    └── internal_lib
        └── nested_lib.sw
```

As `internal_lib` is an internal library, it can be imported into `main.sw` as follows:

- Use the `mod` keyword followed by the library name to make the internal library a dependency
- Use the `use` keyword with a `::` separating the name of the library and the imported item(s)

```sway
mod internal_lib; // Assuming the library name in `internal_lib.sw` is `internal_lib`

use internal_lib::mint;

// `mint` from `internal_library` is now available in this file
```

### External Libraries

External libraries are located outside the main `src` directory as shown below:

```bash
$ tree
.
├── my_project
│   ├── Cargo.toml
│   ├── Forc.toml
│   └─── src
│       └── main.sw
│
└── external_lib
    ├── Cargo.toml
    ├── Forc.toml
    └─── src
        └── lib.sw
```

As `external_lib` is outside the `src` directory of `my_project`, it needs to be added as a dependency in the `Forc.toml` file of `my_project`, by adding the library path in the `dependencies` section as shown below, before it can be imported:

```toml
[dependencies]
external_library = { path = "../external_library" }
```

Once the library dependency is added to the `toml` file, you can import items from it as follows:

- Make sure the item you want imported are declared with the `pub` keyword (if applicable, for instance: `pub fn mint() {}`)
- Use the `use` keyword to selectively import items from the library

```sway
use external_library::mint;

// `mint` from `external_library` is now available in this file
```

Wildcard imports using `*` are supported, but it is generally recommended to use explicit imports where possible.

> **Note**: the standard library is implicitly available to all Forc projects, that is, you are not required to manually specify `std` as an explicit dependency in `Forc.toml`.

## Reference Sway Libraries

The repository [`sway-libs`](https://github.com/FuelLabs/sway-libs/) is a collection of external libraries that you can import and make use of in your Fuel applications. These libraries are meant to be implementations of common use-cases valuable for dapp development.

Some Sway Libraries to try out:

- [Binary Merkle Proof](https://docs.fuel.network/docs/sway-libs/merkle/)
- [Signed Integers](https://github.com/FuelLabs/sway-libs/tree/master/libs/src/signed_integers)
- [Ownership](https://github.com/FuelLabs/sway-libs/tree/master/libs/src/ownership)

### Example

You can import and use a Sway Library such as the [Ownership](https://github.com/FuelLabs/sway-libs/tree/master/libs/src/ownership) library just like any other external library.

```sway
use ownership::Ownership;
```

Once imported, you can use the following basic functionality of the library in your smart contract:

- Declaring an owner
- Changing ownership
- Renouncing ownership
- Ensuring a function may only be called by the owner
