# Storage

## Basic Storage

<!-- This section should explain storage in Sway -->
<!-- storage:example:start -->
When developing a [smart contract](../sway-program-types/smart_contracts.md), you will typically need some sort of persistent storage. In this case, persistent storage, often just called _storage_ in this context, is a place where you can store values that are persisted inside the contract itself. This is in contrast to a regular value in _memory_, which disappears after the contract exits.

Put in conventional programming terms, contract storage is like saving data to a hard drive. That data is saved even after the program that saved it exits. That data is persistent. Using memory is like declaring a variable in a program: it exists for the duration of the program and is non-persistent.

Some basic use cases of storage include declaring an owner address for a contract and saving balances in a wallet.
<!-- storage:example:end -->

### Storage Accesses Via the `storage` Keyword

Declaring variables in storage requires a `storage` block that contains a list of all your variables, their types, and their initial values. The initial value can be any expression that can be evaluated to a constant during compilation, as follows:

```sway
{{#include ../../../../examples/basic_storage_variables/src/main.sw:basic_storage_declaration}}
```

To write into a storage variable, you need to use the `storage` keyword as follows:

```sway
{{#include ../../../../examples/basic_storage_variables/src/main.sw:basic_storage_write}}
```

To read a storage variable, you also need to use the `storage` keyword. You may use `read()` or `try_read()`, however we recommend using `try_read()` for additional safety.

```sway
{{#include ../../../../examples/basic_storage_variables/src/main.sw:basic_storage_read}}
```

### Storing Structs

To store a struct in storage, each variable must be assigned in the `storage` block. This can be either my assigning the fields individually or using a public [constructor](../basics/methods_and_associated_functions.md#constructors) that can be evaluated to a constant during compilation.

```sway
{{#include ../../../../examples/struct_storage_variables/src/main.sw:struct_storage_declaration}}
```

You may write to both fields of a struct and the entire struct as follows:

```sway
{{#include ../../../../examples/struct_storage_variables/src/main.sw:struct_storage_write}}
```

The same applies to reading structs from storage, where both the individual and struct as a whole may be read as follows:

```sway
{{#include ../../../../examples/struct_storage_variables/src/main.sw:struct_storage_read}}
```

### Common Storage Collections

We support the following common storage collections:

- `StorageMap<K, V>`
- `StorageVec<T>`
- `StorageBytes`
- `StorageString`

Please note that these types are not initialized during compilation. This means that if you try to access a key from a storage map before the storage has been set, for example, the call will revert.

Declaring these variables in storage requires a `storage` block as follows:

```sway
{{#include ../../../../examples/advanced_storage_variables/src/main.sw:advanced_storage_declaration}}
```

#### `StorageMaps<K, V>`

Generic storage maps are available in the standard library as `StorageMap<K, V>` which have to be defined inside a `storage` block and allow you to call `insert()` and `get()` to insert values at specific keys and get those values respectively. Refer to [Storage Maps](../common-collections/storage_map.md) for more information about `StorageMap<K, V>`.

To write to a storage map, call either the `insert()` or `try_insert()` functions as follows:

```sway
{{#include ../../../../examples/advanced_storage_variables/src/main.sw:map_storage_write}}
```

The following demonstrates how to read from a storage map:

```sway
{{#include ../../../../examples/advanced_storage_variables/src/main.sw:map_storage_read}}
```

#### `StorageVec<T>`

Generic storage vectors are available in the standard library as `StorageVec<T>` which have to be defined inside a `storage` block and allow you to call `push()` and `pop()` to push and pop values from a vector respectively. Refer to [Storage Vector](../common-collections/storage_vec.md) for more information about `StorageVec<T>`.

The following demonstrates how to write to a `StorageVec<T>`:

```sway
{{#include ../../../../examples/advanced_storage_variables/src/main.sw:vec_storage_write}}
```

The following demonstrates how to read from a `StorageVec<T>`:

```sway
{{#include ../../../../examples/advanced_storage_variables/src/main.sw:vec_storage_read}}
```

#### `StorageBytes`

Storage of `Bytes` is available in the standard library as `StorageBytes` which have to be defined inside a `storage` block. `StorageBytes` cannot be manipulated in the same way a `StorageVec<T>` or `StorageMap<K, V>` can but stores bytes more efficiently thus reducing gas. Only the entirety of a `Bytes` may be read/written to storage. This means any changes would require loading the entire `Bytes` to the heap, making changes, and then storing it once again. If frequent changes are needed, a `StorageVec<u8>` is recommended.

The following demonstrates how to write to a `StorageBytes`:

```sway
{{#include ../../../../examples/advanced_storage_variables/src/main.sw:bytes_storage_write}}
```

The following demonstrates how to read from a `StorageBytes`:

```sway
{{#include ../../../../examples/advanced_storage_variables/src/main.sw:bytes_storage_read}}
```

#### `StorageString`

Storage of `String` is available in the standard library as `StorageString` which have to be defined inside a `storage` block. `StorageString` cannot be manipulated in the same way a `StorageVec<T>` or `StorageMap<K, V>`. Only the entirety of a `String` may be read/written to storage.

The following demonstrates how to write to a `StorageString`:

```sway
{{#include ../../../../examples/advanced_storage_variables/src/main.sw:string_storage_write}}
```

The following demonstrates how to read from a `StorageString`:

```sway
{{#include ../../../../examples/advanced_storage_variables/src/main.sw:string_storage_read}}
```

## Advanced Storage

### Nested Storage Collections

Through the use of `StorageKey`s, you may have nested storage collections such as storing a `StorageString` in a `StorageMap<K, V>`.

For example, here we have a few common nested storage types declared in a `storage` block:

```sway
{{#include ../../../../examples/nested_storage_variables/src/main.sw:nested_storage_declaration}}
```

Please note that storage initialization is needed to do this.

#### Storing a `StorageVec<T>` in a `StorageMap<K, V>`

The following demonstrates how to write to a `StorageVec<T>` that is nested in a `StorageMap<T, V>`:

```sway
{{#include ../../../../examples/nested_storage_variables/src/main.sw:nested_vec_storage_write}}
```

The following demonstrates how to read from a `StorageVec<T>` that is nested in a `StorageMap<T, V>`:

```sway
{{#include ../../../../examples/nested_storage_variables/src/main.sw:nested_vec_storage_read}}
```

#### Storing a `StorageString` in a `StorageMap<K, V>`

The following demonstrates how to write to a `StorageString` that is nested in a `StorageMap<T, V>`:

```sway
{{#include ../../../../examples/nested_storage_variables/src/main.sw:nested_string_storage_write}}
```

The following demonstrates how to read from a `StorageString` that is nested in a `StorageMap<T, V>`:

```sway
{{#include ../../../../examples/nested_storage_variables/src/main.sw:nested_string_storage_read}}
```

#### Storing a `StorageBytes` in a `StorageVec<T>`

The following demonstrates how to write to a `StorageBytes` that is nested in a `StorageVec<T>`:

```sway
{{#include ../../../../examples/nested_storage_variables/src/main.sw:nested_vec_storage_write}}
```

The following demonstrates how to read from a `StorageBytes` that is nested in a `StorageVec<T>`:

```sway
{{#include ../../../../examples/nested_storage_variables/src/main.sw:nested_vec_storage_read}}
```

### Storage Namespace

If you want the values in storage to be positioned differently, for instance to avoid collisions with storage from another contract when loading code, you can use the namespace annotation to add a salt to the slot calculations.

```sway
{{#include ../../../../examples/storage_namespace/src/main.sw:storage_namespace}}
```

### Manual Storage Management

It is possible to leverage FuelVM storage operations directly using the `std::storage::storage_api::write` and `std::storage::storage_api::read` functions provided in the standard library. With this approach, you will have to manually assign the internal key used for storage. An example is as follows:

```sway
{{#include ../../../../examples/storage_example/src/main.sw}}
```

> **Note**: Though these functions can be used for any data type, they should mostly be used for arrays because arrays are not yet supported in `storage` blocks. Note, however, that _all_ data types can be used as types for keys and/or values in `StorageMap<K, V>` without any restrictions.
