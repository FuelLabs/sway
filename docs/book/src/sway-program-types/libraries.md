# Libraries

Libraries in Sway are files used to define new common behavior. The most prominent example of this is the [Sway Standard Library](../introduction/standard_library.html) that is made implicitly available to all Forc projects created using `forc new`.
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

There are two types of Sway libraries, based on their location and how they can be imported.

### Internal Libraries

Internal libraries are located within the main `src` directory with the main program files as shown below:

```bash
$ tree
.
├── Cargo.toml
├── Forc.toml
└── src
    ├── internal_lib.sw
    └── my_library.sw
```


As `internal_lib` is an internal library, it can be imported to use inside `my_library` as follows:
- Use the `dep` keyword followed by the library name to make the internal library a dependancy
- Use the `use` keyword with a `::` separating the name of the library and the import

```sway
library my_library;

dep internal_lib;

use internal_lib::mint;
```

### External Libraries

External libraries are located outside the main `src` directory as shown below:

```bash
$ tree
.
├── my_library
│   ├── Cargo.toml
│   ├── Forc.toml
│   └─── src
│       └── lib.sw
│
└── external_lib
    ├── Cargo.toml
    ├── Forc.toml
    └─── src
        └── lib.sw
```
        
As `external_lib` is outside the main `src` directory of `my_library`, it needs to be added as a dependancy in `Forc.toml` of `my_library` by adding the library path in the `dependancies` section as below in order to be imported:

```toml
[dependencies]
external_library = { path = "../external_library" }
```

Once the dependancy is added, you can import external libraries as follows:
- Make sure the imports you want to add have the `pub` keyword (for instance: `pub fn mint() {}`)
- Use the `use` keyword to selectively import items from the library

```sway
use external_library::mint;

// `mint` from `external_library` is now available in this file
```

Wildcard imports using `*` are supported, but it is generally recommended to use explicit imports where possible.

> **Note**: the standard library is implicitly available to all Forc projects, that is, you are not required to manually specify `std` as an explicit dependency in `Forc.toml`.

## Sway Libraries

The repository [`sway-libs`](https://github.com/FuelLabs/sway-libs/tree/master/sway_libs/) is a collection of external libraries that you can import and make use of in your Fuel applications. These libraries are meant to be learning references of common use-cases valuable for dapp development.

Some Sway Libraries to try out:
- [Binary Merkle Proof](https://github.com/FuelLabs/sway-libs/blob/master/sway_libs/src/merkle_proof)
- [Non-Fungible Token](https://github.com/FuelLabs/sway-libs/tree/master/sway_libs/src/nft)
- [String](https://github.com/FuelLabs/sway-libs/blob/master/sway_libs/src/string)
- [Signed Integers](https://github.com/FuelLabs/sway-libs/blob/master/sway_libs/src/signed_integers)
- [Unsigned Fixed Point Number](https://github.com/mehtavishwa30/sway-libs/blob/master/sway_libs/src/fixed_point/ufp)
- [StorageMapVec](https://github.com/mehtavishwa30/sway-libs/blob/master/sway_libs/src/storagemapvec)

### Example

You can import and use a Sway Library such as the [NFT](https://github.com/FuelLabs/sway-libs/tree/master/sway_libs/src/nft) library just like any other external library.

```sway
use sway_libs::nft::{
    mint,
    transfer,
    owner_of,
    approve,
};
```
Once imported, you can use the following basic functionality of the library in your smart contract:
- Minting tokens
- Transfering tokens
- Retrieving owner of a token
- Approving users to transfer a token
