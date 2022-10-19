# StorageMap

A `StorageMap`, a.k.a. a hash table, is a structure which associates a value `v` with a key `k`. The key is used to find the position in the table (memory) where the value is stored. 

The benefit of a hash table is that no matter where the value is in the table the computation required to find the location of that value is constant i.e. it has an order of 1 `O(1)`.

Sway provides a flexible `StorageMap` because it uses [generics](../../language/generics/index.md) for both `k` & `v` with the caveat that `k` and `v` have to be a single value. The value can be a struct, tuple, array etc. therefore if you'd like to have a complex `k` or `v` then the data needs to be wrapped into a single type.

## Declaration

To use a `StorageMap` we need to import it from the standard library and while we're at it we'll import the `msg_sender()` so that we can use it in the following section.

After the import we initialize our `StorageMap` as described in the [initialization](init.md) section.

```sway
{{#include ../../../code/operations/storage/storage_map/src/main.sw:initialization}}
```

There are two `storage` variables: `balance` & `user`. `balance` takes a single value as the key while `user` wraps two values into a [tuple](../../language/built-ins/tuples.md) and uses that as a key.

## Usage

When dealing with storage we have two options, we can either read from or write to storage. In both cases we must use a [storage annotation](../../language/annotations/attributes/storage.md) to indicate the purity of the function.

When referencing a variable in storage we must explicitly indicate that the variable comes from storage and not a local scope. 

This is done via the syntax `storage.variable_name`.

### Reading from Storage

Retrieving data from a storage variable is done through the `.get(key)` method. That is to say that we state which storage variable we would like to read from and append `.get()` to the end while providing the key for the data that we want to retrieve.

In this example we wrap the `Identity` of the caller with their provided `id` into a [tuple](../../language/built-ins/tuples.md) and use that as the key.

```sway
{{#include ../../../code/operations/storage/storage_map/src/main.sw:reading_from_storage}}
```

### Writing to Storage

Writing to storage is similar to [reading](#reading-from-storage). The difference is that we use a different method `.insert(key, value)`.

In this example we retrieve the balance of the caller and then increment their balance by 1.

```sway
{{#include ../../../code/operations/storage/storage_map/src/main.sw:writing_to_storage}}
```