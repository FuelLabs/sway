# Storage Maps

Another important common collection is the storage map. The type `StorageMap<K, V>` from the standard library stores a mapping of keys of type `K` to values of type `V` using a hashing function, which determines how it places these keys and values into _storage slots_. This is similar to [Rust's `HashMap<K, V>`](https://doc.rust-lang.org/std/collections/struct.HashMap.html) but with a few differences.

Storage maps are useful when you want to look up data not by using an index, as you can with vectors, but by using a key that can be of any type. For example, when building a ledger-based sub-currency smart contract, you could keep track of the balance of each wallet in a storage map in which each key is a wallet’s `Address` and the values are each wallet’s balance. Given an `Address`, you can retrieve its balance.

Similarly to `StorageVec<T>`, `StorageMap<K, V>` can only be used in a contract because only contracts are allowed to access persistent storage.

In order to use `StorageMap<K, V>`, you must first import `StorageMap` as follows:

```sway
{{#include ../../../examples/storage_map/src/main.sw:storage_map_import}}
```

## Creating a New Storage Map

To create a new empty storage map, we have to declare the map in a `storage` block as follows:

```sway
{{#include ../../../examples/storage_map/src/main.sw:storage_map_decl}}
```

Just like any other storage variable, two things are required when declaring a `StorageMap`: a type annotation and an initializer. The initializer is just an empty struct of type `StorageMap` because `StorageMap<K, V>` itself is an empty struct! Everything that is interesting about `StorageMap<K, V>` is implemented in its methods.

Storage maps, just like `Vec<T>` and `StorageVec<T>`, are implemented using generics which means that the `StorageMap<K, V>` type provided by the standard library can map keys of any type `K` to values of any type `V`. In the example above, we’ve told the Sway compiler that the `StorageMap<K, V>` in `map` will map keys of type `Address` to values of type `u64`.

## Updating a Storage Map

To insert key-value pairs into a storage map, we can use the `insert` method, as shown below:

```sway
{{#include ../../../examples/storage_map/src/main.sw:storage_map_insert}}
```

Note two details here. First, in order to use `insert`, we need to first access the storage map using the `storage` keyword. Second, because `insert` requires _writing_ into storage, a `#[storage(write)]` annotation is required on the ABI function that calls `insert`.

> **Note**
> The storage annotation is also required for any private function defined in the contract that tries to insert into the map.

<!-- markdownlint-disable-line MD028 -->
> **Note**
> There is no need to add the `mut` keyword when declaring a `StorageMap<K, V>`. All storage variables are mutable by default.

## Accessing Values in a Storage Map

We can get a value out of the storage map by providing its `key` to the `get` method, as shown below:

```sway
{{#include ../../../examples/storage_map/src/main.sw:storage_map_get}}
```

Here, `value1` will have the value that's associated with the first address, and the result will be `42`. You might expect `get` to return an `Option<V>` where the return value would be `None` if the value does not exist. However, that is not case for `StorageMap`. In fact, storage maps have no way of knowing whether `insert` has been called with a given key or not as it would be too expensive to keep track of that information. Instead, a default value whose byte-representation is all zeros is returned if `get` is called with a key that has no value in the map. Note that each type interprets that default value differently:

* The default value for a `bool` is `false`.
* The default value for a integers is `0`.
* The default value for a `b256` is `0x0000000000000000000000000000000000000000000000000000000000000000`.
* The default value for a `str[n]` is a string of `Null` characters.
* The default value for a tuple is a tuple of the default values of its components.
* The default value for a struct is a struct of the default values of its components.
* The default value for an enum is an instance of its first variant containing the default for its associated value.

## Storage maps with multiple keys

You might find yourself needing a `StorageMap<K1, V1>` where the type `V1` is itself another `StorageMap<K2, V2>`. This is not allowed in Sway. The right approach is to use a single `StorageMap<K, V>` where `K` is a tuple `(K1, K2)`. For example:

```sway
{{#include ../../../examples/storage_map/src/main.sw:storage_map_tuple_key}}
```

## Limitations

It is not currently allowed to have a `StorageMap<K, V>` as a component of a complex type such as a struct or an enum. For example, the code below is not legal:

```sway
Struct Wrapper {
    map1: StorageMap<u64, u64>,
    map2: StorageMap<u64, u64>,
}

storage {
    w: Wrapper
}
...

storage.w.map1.insert(..);
```
