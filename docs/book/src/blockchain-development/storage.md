# Storage

<!-- This section should explain storage in Sway -->
<!-- storage:example:start -->
When developing a [smart contract](../sway-program-types/smart_contracts.md), you will typically need some sort of persistent storage. In this case, persistent storage, often just called _storage_ in this context, is a place where you can store values that are persisted inside the contract itself. This is in contrast to a regular value in _memory_, which disappears after the contract exits.

Put in conventional programming terms, contract storage is like saving data to a hard drive. That data is saved even after the program that saved it exits. That data is persistent. Using memory is like declaring a variable in a program: it exists for the duration of the program and is non-persistent.

Some basic use cases of storage include declaring an owner address for a contract and saving balances in a wallet.
<!-- storage:example:end -->

## Storage Accesses Via the `storage` Keyword

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

## Storing Structs

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

## Common Storage Collections

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

### `StorageMaps<K, V>`

Generic storage maps are available in the standard library as `StorageMap<K, V>` which have to be defined inside a `storage` block and allow you to call `insert()` and `get()` to insert values at specific keys and get those values respectively. Refer to [Storage Maps](../common-collections/storage_map.md) for more information about `StorageMap<K, V>`.

**Warning** While the `StorageMap<K, V>` is currently included in the prelude, to use it the `Hash` trait must still be imported. This is a known issue and will be resolved.

```sway
{{#include ../../../../examples/advanced_storage_variables/src/main.sw:temp_hash_import}}
```

To write to a storage map, call either the `insert()` or `try_insert()` functions as follows:

```sway
{{#include ../../../../examples/advanced_storage_variables/src/main.sw:map_storage_write}}
```

The following demonstrates how to read from a storage map:

```sway
{{#include ../../../../examples/advanced_storage_variables/src/main.sw:map_storage_read}}
```

### `StorageVec<T>`

Generic storage vectors are available in the standard library as `StorageVec<T>` which have to be defined inside a `storage` block and allow you to call `push()` and `pop()` to push and pop values from a vector respectively. Refer to [Storage Vector](../common-collections/storage_vec.md) for more information about `StorageVec<T>`.

The following demonstrates how to import `StorageVec<T>`:

```sway
{{#include ../../../../examples/advanced_storage_variables/src/main.sw:storage_vec_import}}
```

> **NOTE**: When importing the `StorageVec<T>`, please be sure to use the glob operator: `use std::storage::storage_vec::*`.

The following demonstrates how to write to a `StorageVec<T>`:

```sway
{{#include ../../../../examples/advanced_storage_variables/src/main.sw:vec_storage_write}}
```

The following demonstrates how to read from a `StorageVec<T>`:

```sway
{{#include ../../../../examples/advanced_storage_variables/src/main.sw:vec_storage_read}}
```

### `StorageBytes`

Storage of `Bytes` is available in the standard library as `StorageBytes` which have to be defined inside a `storage` block. `StorageBytes` cannot be manipulated in the same way a `StorageVec<T>` or `StorageMap<K, V>` can but stores bytes more efficiently thus reducing gas. Only the entirety of a `Bytes` may be read/written to storage. This means any changes would require loading the entire `Bytes` to the heap, making changes, and then storing it once again. If frequent changes are needed, a `StorageVec<u8>` is recommended.

The following demonstrates how to import `StorageBytes`:

```sway
{{#include ../../../../examples/advanced_storage_variables/src/main.sw:storage_bytes_import}}
```

> **NOTE**: When importing the `StorageBytes`, please be sure to use the glob operator: `use std::storage::storage_bytes::*`.

The following demonstrates how to write to a `StorageBytes`:

```sway
{{#include ../../../../examples/advanced_storage_variables/src/main.sw:bytes_storage_write}}
```

The following demonstrates how to read from a `StorageBytes`:

```sway
{{#include ../../../../examples/advanced_storage_variables/src/main.sw:bytes_storage_read}}
```

### `StorageString`

Storage of `String` is available in the standard library as `StorageString` which have to be defined inside a `storage` block. `StorageString` cannot be manipulated in the same way a `StorageVec<T>` or `StorageMap<K, V>`. Only the entirety of a `String` may be read/written to storage.

The following demonstrates how to import `StorageString`:

```sway
{{#include ../../../../examples/advanced_storage_variables/src/main.sw:storage_string_import}}
```

> **NOTE**: When importing the `StorageString`, please be sure to use the glob operator: `use std::storage::storage_string::*`.

The following demonstrates how to write to a `StorageString`:

```sway
{{#include ../../../../examples/advanced_storage_variables/src/main.sw:string_storage_write}}
```

The following demonstrates how to read from a `StorageString`:

```sway
{{#include ../../../../examples/advanced_storage_variables/src/main.sw:string_storage_read}}
```

## Advanced Storage

For more advanced storage techniques please refer to the [Advanced Storage](../advanced/advanced_storage.md) page.
