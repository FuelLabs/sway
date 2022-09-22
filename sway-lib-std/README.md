# Sway Standard Library

The Sway Standard Library is the foundation of portable Sway software, a set of minimal shared abstractions for the broader Sway ecosystem. It offers core types, like `Result<T, E>` and `Option<T>`, library-defined operations on language primitives, native asset management, blockchain contextual operations, access control, storage management, and support for types from other VMs, among many other things.

## Usage

The standard library is made implicitly available to all Forc projects created using `forc new`. In other words, it is not required to manually specify `std` as an explicit dependency. Forc will automagically use the version of `std` that matches its version.

Importing items from the standard library can be done using the `use` keyword, just as importing items from any other Sway library. For example:

```sway
use std::storage::StorageMap;
```

This imports the `StorageMap` type into the current namespace.

The standard library comes with a "[prelude](https://github.com/FuelLabs/sway/blob/master/sway-lib-std/src/prelude.sw)" which is a list of things that Sway automatically imports into every Sway program. It's kept as small as possible, and is focused on things which are used in almost every single Sway program.
