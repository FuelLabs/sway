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

Generic storage maps are available in the standard library as `StorageMap<K, V>` which have to be defined inside a `storage` block and allow you to call `insert()` and `get()` to insert values at specific keys and get those values respectively. For example:

```sway
{{#include ../../../examples/storage_map/src/main.sw}}
```

Because storage maps have to be defined inside a `storage` block, the `storage` keyword is required to access the map itself and then access the appropriate method.

> **Note**: Calling `get(k)` for some key `k` that does not exist in the map (i.e. `insert()` hasn't been called with key `k` yet) returns zero. This is because the FuelVM initializes all storage slots to zero.

## Manual Storage Management

Outside of the newer experimental `storage` syntax which is being stabalized, you can leverage FuelVM storage operations using the `store` and `get` methods provided in the standard library (`std`). Which currently works with primitive types.

With this approach you will have to manually assign the internal key used for storage.

An example is as follows:

```sway
{{#include ../../../examples/storage_example/src/main.sw}}
```

> **Note**: if you are looking to store non-primitive types (e.g. b256), please refer to [this issue](https://github.com/FuelLabs/sway/issues/1229).
