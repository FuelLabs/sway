# Standard Library

Similar to Rust, Sway comes with its own standard library.

The Sway Standard Library is the foundation of portable Sway software, a set of minimal shared abstractions for the broader Sway ecosystem. It offers core types, like `Result<T, E>` and `Option<T>`, library-defined operations on language primitives, native asset management, blockchain contextual operations, access control and storage management, among many other things.

The standard library is made implicitly available to all Forc projects created using [`forc init`](../forc/commands/forc_init.md). Importing items from the standard library can be done using the `use` keyword. Example:

```sway
use std::address::Address;

## Using the Standard Library

Aside from the references to the standard library accross this book, you may also read the `std` library project directly [here](https://github.com/FuelLabs/sway/tree/master/sway-lib-std).

