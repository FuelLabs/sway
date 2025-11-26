# StorageVec

A `StorageVec` is a vector that permanently stores its data in `storage`. It replicates the functionality of a regular vector however its data is not stored contiguously because it utilizes hashing and [generics](../../../language/generics/index.md) to find a location to store the value `T`.

There is a number of methods in the [standard library](https://github.com/FuelLabs/sway/blob/master/sway-lib-std/src/storage.sw) however we will take a look at pushing and retrieving data.

## Declaration

To use a `StorageVec` we need to import it from the [standard library](https://github.com/FuelLabs/sway/tree/master/sway-lib-std) and while we're at it we'll import the `msg_sender()` so that we can use it in the following section.

After the import we initialize our `StorageVec` as described in the [initialization](../init.md) section.

```sway
{{#include ../../../../code/operations/storage/storage_vec/src/main.sw:initialization}}
```

There are two `storage` variables: `balance` & `user`. `balance` takes a single value while `user` wraps two values into a [tuple](../../../language/built-ins/tuples.md).

## Reading from Storage

Retrieving data from a storage variable is done through the `.get(index)` method. That is to say that we state which index by specifying it inside `.get()` and appending that to the end of the storage variable.

In this example we look at how we can retrieve a single value `balance` and how we can unpack multiple values from `user`.

```sway
{{#include ../../../../code/operations/storage/storage_vec/src/main.sw:reading_from_storage}}
```

## Writing to Storage

Writing to storage is similar to [reading](#reading-from-storage). The difference is that we use a different method `.push(value)` and we use the `read` keyword because the implementation reads the length of the vector to determine where to store the value.

In this example we insert a [tuple](../../../language/built-ins/tuples.md) containing the [`Identity`](../../namespace/identity.md) of the caller and some `id` into the vector.

```sway
{{#include ../../../../code/operations/storage/storage_vec/src/main.sw:writing_to_storage}}
```
