# Advanced Storage

## Nested Storage Collections

Through the use of `StorageKey`s, you may have nested storage collections such as storing a `StorageString` in a `StorageMap<K, V>`.

For example, here we have a few common nested storage types declared in a `storage` block:

```sway
{{#include ../../../../examples/nested_storage_variables/src/main.sw:nested_storage_declaration}}
```

Please note that storage initialization is needed to do this.

> **NOTE**: When importing a storage type, please be sure to use the glob operator i.e. `use std::storage::storage_vec::*`.

### Storing a `StorageVec<T>` in a `StorageMap<K, V>`

The following demonstrates how to write to a `StorageVec<T>` that is nested in a `StorageMap<T, V>`:

```sway
{{#include ../../../../examples/nested_storage_variables/src/main.sw:nested_vec_storage_write}}
```

The following demonstrates how to read from a `StorageVec<T>` that is nested in a `StorageMap<T, V>`:

```sway
{{#include ../../../../examples/nested_storage_variables/src/main.sw:nested_vec_storage_read}}
```

### Storing a `StorageString` in a `StorageMap<K, V>`

The following demonstrates how to write to a `StorageString` that is nested in a `StorageMap<T, V>`:

```sway
{{#include ../../../../examples/nested_storage_variables/src/main.sw:nested_string_storage_write}}
```

The following demonstrates how to read from a `StorageString` that is nested in a `StorageMap<T, V>`:

```sway
{{#include ../../../../examples/nested_storage_variables/src/main.sw:nested_string_storage_read}}
```

### Storing a `StorageBytes` in a `StorageVec<T>`

The following demonstrates how to write to a `StorageBytes` that is nested in a `StorageVec<T>`:

```sway
{{#include ../../../../examples/nested_storage_variables/src/main.sw:nested_vec_storage_write}}
```

The following demonstrates how to read from a `StorageBytes` that is nested in a `StorageVec<T>`:

```sway
{{#include ../../../../examples/nested_storage_variables/src/main.sw:nested_vec_storage_read}}
```

## Storage Namespace

If you want the values in storage to be positioned differently, for instance to avoid collisions with storage from another contract when loading code, you can use the namespace annotation to add a salt to the slot calculations.

```sway
{{#include ../../../../examples/storage_namespace/src/main.sw:storage_namespace}}
```

## Manual Storage Management

It is possible to leverage FuelVM storage operations directly using the `std::storage::storage_api::write` and `std::storage::storage_api::read` functions provided in the standard library. With this approach, you will have to manually assign the internal key used for storage. An example is as follows:

```sway
{{#include ../../../../examples/storage_example/src/main.sw}}
```

> **Note**: Though these functions can be used for any data type, they should mostly be used for arrays because arrays are not yet supported in `storage` blocks. Note, however, that _all_ data types can be used as types for keys and/or values in `StorageMap<K, V>` without any restrictions.
