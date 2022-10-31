# StorageMap

A `StorageMap`, a.k.a. a hash table, is a structure which associates a value `v` with a key `k`. The key is used to find the position in the table (memory) where the value is stored. 

The benefit of a hash table is that no matter where the value is in the table the computation required to find the location of that value is constant i.e. it has an order of 1 `O(1)`.

Sway provides a flexible `StorageMap` because it uses [generics](../../../language/generics/index.md) for both `k` & `v` with the caveat that `k` and `v` have to be a single value. The value can be a [struct](../../../language/built-ins/structs.md), [tuple](../../../language/built-ins/tuples.md), [array](../../../language/built-ins/arrays.md) etc. therefore if you'd like to have a complex `k` or `v` then the data needs to be wrapped into a single type.

## Declaration

The `StorageMap` type is included in the [prelude](../../../misc/prelude.md) therefore we do not need to import it. We'll be using `msg_sender()` in the subsequent section so we'll import that here.

After the import we initialize our `StorageMap` as described in the [initialization](../init.md) section.

```sway
{{#include ../../../../code/operations/storage/storage_map/src/main.sw:initialization}}
```

There are two `storage` variables: `balance` & `user`. `balance` takes a single value as the key while `user` wraps two values into a [tuple](../../../language/built-ins/tuples.md) and uses that as a key.

## Reading from Storage

Retrieving data from a storage variable is done through the `.get(key)` method. That is to say that we state which storage variable we would like to read from and append `.get()` to the end while providing the key for the data that we want to retrieve.

In this example we wrap the [`Identity`](../../namespace/identity.md) of the caller with their provided `id` into a [tuple](../../../language/built-ins/tuples.md) and use that as the key.

```sway
{{#include ../../../../code/operations/storage/storage_map/src/main.sw:reading_from_storage}}
```

## Writing to Storage

Writing to storage is similar to [reading](#reading-from-storage). The difference is that we use a different method `.insert(key, value)`.

In this example we retrieve the balance of the caller and then increment their balance by 1.

```sway
{{#include ../../../../code/operations/storage/storage_map/src/main.sw:writing_to_storage}}
```
