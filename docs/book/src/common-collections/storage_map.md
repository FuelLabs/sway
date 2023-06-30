# Storage Maps

Another important common collection is the storage map.

<!-- This section should explain storage maps in Sway -->
<!-- storage_map:example:start -->
The type `StorageMap<K, V>` from the standard library stores a mapping of keys of type `K` to values of type `V` using a hashing function, which determines how it places these keys and values into _storage slots_. This is similar to [Rust's `HashMap<K, V>`](https://doc.rust-lang.org/std/collections/struct.HashMap.html) but with a few differences.

Storage maps are useful when you want to look up data not by using an index, as you can with vectors, but by using a key that can be of any type. For example, when building a ledger-based sub-currency smart contract, you could keep track of the balance of each wallet in a storage map in which each key is a wallet’s `Address` and the values are each wallet’s balance. Given an `Address`, you can retrieve its balance.

Similarly to `StorageVec<T>`, `StorageMap<K, V>` can only be used in a contract because only contracts are allowed to access persistent storage.

`StorageMap<T>` is included in the [standard library prelude](../introduction/standard_library.md#standard-library-prelude) which means that there is no need to import it manually.
<!-- storage_map:example:end -->

## Creating a New Storage Map

To create a new empty storage map, we have to declare the map in a `storage` block as follows:

```sway
{{#include ../../../../examples/storage_map/src/main.sw:storage_map_decl}}
```

<!-- This section should explain how to implement storage maps in Sway -->
<!-- use_storage_maps:example:start -->
Just like any other storage variable, two things are required when declaring a `StorageMap`: a type annotation and an initializer. The initializer is just an empty struct of type `StorageMap` because `StorageMap<K, V>` itself is an empty struct! Everything that is interesting about `StorageMap<K, V>` is implemented in its methods.

Storage maps, just like `Vec<T>` and `StorageVec<T>`, are implemented using generics which means that the `StorageMap<K, V>` type provided by the standard library can map keys of any type `K` to values of any type `V`. In the example above, we’ve told the Sway compiler that the `StorageMap<K, V>` in `map` will map keys of type `Address` to values of type `u64`.
<!-- use_storage_maps:example:end -->

## Updating a Storage Map

<!-- This section should explain how to update storage maps in Sway -->
<!-- update_storage_maps:example:start -->
To insert key-value pairs into a storage map, we can use the `insert` method.
<!-- update_storage_maps:example:end -->

For example:

```sway
{{#include ../../../../examples/storage_map/src/main.sw:storage_map_insert}}
```

Note two details here. First, in order to use `insert`, we need to first access the storage map using the `storage` keyword. Second, because `insert` requires _writing_ into storage, a `#[storage(write)]` annotation is required on the ABI function that calls `insert`.

> **Note**
> The storage annotation is also required for any private function defined in the contract that tries to insert into the map.

<!-- markdownlint-disable-line MD028 -->
> **Note**
> There is no need to add the `mut` keyword when declaring a `StorageMap<K, V>`. All storage variables are mutable by default.

## Accessing Values in a Storage Map

<!-- This section should explain how to access storage map values in Sway -->
<!-- access_storage_maps:example:start -->
We can get a value out of the storage map by providing its `key` to the `get` method.
<!-- access_storage_maps:example:end -->

For example:

```sway
{{#include ../../../../examples/storage_map/src/main.sw:storage_map_get}}
```

Here, `value1` will have the value that's associated with the first address, and the result will be `42`. The `get` method returns an `Option<V>`; if there’s no value for that key in the storage map, `get` will return `None`. This program handles the `Option` by calling `unwrap_or` to set `value1` to zero if `map` doesn't have an entry for the key.

## Storage Maps with Multiple Keys

Maps with multiple keys can be implemented using tuples as keys. For example:

```sway
{{#include ../../../../examples/storage_map/src/main.sw:storage_map_tuple_key}}
```

## Nested Storage Maps

It is possible to nest storage maps as follows:

```sway
{{#include ../../../../examples/storage_map/src/main.sw:storage_map_nested}}
```

The nested map can then be accessed as follows:

```sway
{{#include ../../../../examples/storage_map/src/main.sw:storage_map_nested_access}}
```
