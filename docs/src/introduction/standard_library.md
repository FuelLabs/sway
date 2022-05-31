# Standard Library

Similar to Rust, Sway comes with its own standard library.

The Sway Standard Library is the foundation of portable Sway software, a set of minimal shared abstractions for the broader Sway ecosystem. It offers core types, like `Result<T, E>` and `Option<T>`, library-defined operations on language primitives, native asset management, blockchain contextual operations, access control, storage management, and support for types from other VMs, among many other things.

The entire Sway standard library is a Forc project called `std`, and is available directly here: <https://github.com/FuelLabs/sway/tree/master/sway-lib-std> (navigate to the appropriate tagged release if the latest `master` is not compatible).

## Using the Standard Library

The standard library is made implicitly available to all Forc projects created using [`forc new`](../forc/commands/forc_new.md). In other words, it is not required to manually specify `std` as an explicit dependency. Forc will automagically use the version of `std` that matches its version.

Importing items from the standard library can be done using the `use` keyword, just as importing items from any Sway project. For example:

```sway
use std::address::Address;
```

This imports the `Address` type into the current namespace.
