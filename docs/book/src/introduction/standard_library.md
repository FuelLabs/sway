# Standard Library

<!-- This section should explain what the std-lib is -->
<!-- std_lib:example:start -->
Similar to Rust, Sway comes with its own standard library.

The Sway Standard Library is the foundation of portable Sway software, a set of minimal shared abstractions for the broader Sway ecosystem. It offers core types, like `Result<T, E>` and `Option<T>`, library-defined operations on language primitives, native asset management, blockchain contextual operations, access control, storage management, and support for types from other VMs, among many other things.
<!-- std_lib:example:end -->

The entire Sway standard library is a Forc project called `std`, and is available directly [here](https://github.com/FuelLabs/sway/tree/master/sway-lib-std). Navigate to the appropriate tagged release if the latest `master` is not compatible. You can find the latest `std` documentation [here](https://fuellabs.github.io/sway/master/std/).

## Using the Standard Library

The standard library is made implicitly available to all Forc projects created using [`forc new`](../forc/commands/forc_new.md). In other words, it is not required to manually specify `std` as an explicit dependency. Forc will automatically use the version of `std` that matches its version.

Importing items from the standard library can be done using the `use` keyword, just as importing items from any Sway project. For example:

```sway
use std::storage::storage_vec::*;
```

This imports the `StorageVec` type into the current namespace.

## Standard Library Prelude

<!-- This section should explain what the std-lib prelude is -->
<!-- prelude:example:start -->
Sway comes with a variety of things in its standard library. However, if you had to manually import every single thing that you used, it would be very verbose. But importing a lot of things that a program never uses isn't good either. A balance needs to be struck.

The prelude is the list of things that Sway automatically imports into every Sway program. It's kept as small as possible, and is focused on things which are used in almost every single Sway program.

The current version of the prelude lives in [`std::prelude`](https://github.com/FuelLabs/sway/blob/master/sway-lib-std/src/prelude.sw), and re-exports the following:

- [`std::address::Address`](https://github.com/FuelLabs/sway/blob/master/sway-lib-std/src/address.sw), a wrapper around the `b256` type representing a wallet address.
- [`std::contract_id::ContractId`](https://github.com/FuelLabs/sway/blob/master/sway-lib-std/src/contract_id.sw), a wrapper around the `b256` type representing the ID of a contract.
- [`std::identity::Identity`](https://github.com/FuelLabs/sway/blob/master/sway-lib-std/src/identity.sw), an enum with two possible variants: `Address: Address` and `ContractId: ContractId`.
- [`std::vec::Vec`](https://github.com/FuelLabs/sway/blob/master/sway-lib-std/src/vec.sw), a growable, heap-allocated vector.
- [`std::storage::storage_key::*`](https://github.com/FuelLabs/sway/blob/master/sway-lib-std/src/storage/storage_key.sw), contains the API for accessing a `core::storage::StorageKey` which describes a location in storage.
- [`std::storage::storage_map::*`](https://github.com/FuelLabs/sway/blob/master/sway-lib-std/src/storage/storage_map.sw), a key-value mapping in contract storage.
- [`std::option::Option`](https://github.com/FuelLabs/sway/blob/master/sway-lib-std/src/option.sw), an enum which expresses the presence or absence of a value.
- [`std::result::Result`](https://github.com/FuelLabs/sway/blob/master/sway-lib-std/src/result.sw), an enum for functions that may succeed or fail.
- [`std::assert::assert`](https://github.com/FuelLabs/sway/blob/master/sway-lib-std/src/assert.sw), a function that reverts the VM if the condition provided to it is `false`.
- [`std::assert::assert_eq`](https://github.com/FuelLabs/sway/blob/master/sway-lib-std/src/assert.sw), a function that reverts the VM and logs its two inputs `v1` and `v2` if the condition `v1` == `v2` is `false`.
- [`std::assert::assert_ne`](https://github.com/FuelLabs/sway/blob/master/sway-lib-std/src/assert.sw), a function that reverts the VM and logs its two inputs `v1` and `v2` if the condition `v1` != `v2` is `false`.
- [`std::revert::require`](https://github.com/FuelLabs/sway/blob/master/sway-lib-std/src/revert.sw), a function that reverts the VM and logs a given value if the condition provided to it is `false`.
- [`std::revert::revert`](https://github.com/FuelLabs/sway/blob/master/sway-lib-std/src/revert.sw), a function that reverts the VM.
- [`std::logging::log`](https://github.com/FuelLabs/sway/blob/master/sway-lib-std/src/logging.sw), a function that logs arbitrary stack types.
- [`std::auth::msg_sender`](https://github.com/FuelLabs/sway/blob/master/sway-lib-std/src/auth.sw), a function that gets the `Identity` from which a call was made.
<!-- prelude:example:end -->
