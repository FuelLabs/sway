# Storage

When developing a [smart contract](../sway-program-types/smart_contracts.md), you will typically need some sort of persistent storage. In this case, persistent storage, often just called _storage_ in this context, is a place where you can store values that are persisted inside the contract itself. This is in contrast to a regular value in _memory_, which disappears after the contract exits.

Put in conventional programming terms, contract storage is like saving data to a hard drive. That data is saved even after the program which saved it exits. That data is persistent. Using memory is like declaring a variable in a program: it exists for the duration of the program and is non-persistent.

Some basic use cases of storage include declaring an owner address for a contract and saving balances in a wallet.

## Storage Accesses Via the `storage` Keyword

Declaring variables in storage requires a `storage` declaration that contains a list of all your variables and their types as follows:

```sway
storage {
    var1: Type1,
    var2: Type2,
    ...
}
```

To write into a storage variable, you need to use the `storage` keyword as follows:

```sway
storage.var1 = v;
```

To read a storage variable, you also need to use the `storage` keyword as follows:

```sway
let v = storage.var1;
```

Notes:

* The only types currently supported by the syntax above are integers, Booleans, and structs.
* Storage, in general, is still work-in-progress and so, its use model may change in the future.

## Storage Maps

Generic storage maps are available in the standard library as `StorageMap<K, V>` which have to be defined inside a `storage` block and allow you to call `insert()` and `get()` to insert values at specific keys and get those values respectively. Storage maps also have to be initialized using `new()`. For example:

```sway
{{#include ../../../examples/storage_map/src/main.sw}}
```

There are three important components to correctly using a storage map:

* Declaring the storage map with your desired data types inside a `storage` block:

```sway
storage {
    map1: Storage<u64, u64>,
    // Other storage items
}
```

* Initializing the storage map exactly once using `new()` in some contract method before using it:

```sway
fn init() {
    storage.map1 = ~StorageMap::new::<u64, u64>();
    // Other items to initialize
}
```

The contract method that calls `new()` has to be called from an external context (such as you Rust SDK test) before you can actually use `insert()` and `get()` correctly.

* Calling `insert()` and `get()` as needed:

```sway
storage.map1.insert(42, 99);
let value_in_key_42 = storage.map1.get(42);
```

## Manual Storage Management

Outside of the newer experimental `storage` syntax which is being stabalized, you can leverage FuelVM storage operations using the `store` and `get` methods provided in the standard library (`std`). Which currently works with primitive types.

With this approach you will have to manually assign the internal key used for storage.

An example is as follows:

```sway
{{#include ../../../examples/storage_example/src/main.sw}}
```

> **Note**: if you are looking to store non-primitive types (e.g. b256), please refer to [this issue](https://github.com/FuelLabs/sway/issues/1229).
