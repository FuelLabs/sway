# Storage Vectors

The second collection type we’ll look at is `StorageVec<T>`. Just like vectors on the heap (i.e. `Vec<T>`), storage vectors allow you to store more than one value in a single data structure where each value is assigned an index and can only store values of the same type. However, unlike `Vec<T>`, the elements of a `StorageVec` are stored in _persistent storage_, and consecutive elements are not necessarily stored in storage slots that have consecutive keys.

In order to use `StorageVec<T>`, you must first import `StorageVec` as follows:

```sway
{{#include ../../../../examples/storage_vec/src/main.sw:storage_vec_import}}
```

Another major difference between `Vec<T>` and `StorageVec<T>` is that `StorageVec<T>` can only be used in a contract because only contracts are allowed to access persistent storage.

## Creating a New Storage Vector

To create a new empty storage vector, we have to declare the vector in a `storage` block as follows:

```sway
{{#include ../../../../examples/storage_vec/src/main.sw:storage_vec_decl}}
```

Just like any other storage variable, two things are required when declaring a `StorageVec`: a type annotation and an initializer. The initializer is just an empty struct of type `StorageVec` because `StorageVec<T>` itself is an empty struct! Everything that is interesting about `StorageVec<T>` is implemented in its methods.

Storage vectors, just like `Vec<T>`, are implemented using generics which means that the `StorageVec<T>` type provided by the standard library can hold any type. When we create a storage vector to hold a specific type, we can specify the type within angle brackets. In the example above, we’ve told the Sway compiler that the `StorageVec<T>` in `v` will hold elements of the `u64` type.

## Updating a Storage Vector

To add elements to a storage vector, we can use the `push` method, as shown below:

```sway
{{#include ../../../../examples/storage_vec/src/main.sw:storage_vec_push}}
```

Note two details here. First, in order to use `push`, we need to first access the vector using the `storage` keyword. Second, because `push` requires accessing storage, a `storage` annotation is required on the ABI function that calls `push`. While it may seem that `#[storage(write)]` should be enough here, the `read` annotation is also required because each call to `push` requires _reading_ (and then updating) the length of the storage vector which is also stored in persistent storage.

> **Note**
> The storage annotation is also required for any private function defined in the contract that tries to push into the vector.

<!-- markdownlint-disable-line MD028 -->
> **Note**
> There is no need to add the `mut` keyword when declaring a `StorageVec<T>`. All storage variables are mutable by default.

## Reading Elements of Storage Vectors

To read a value stored in a vector at a particular index, you can use the `get` method as shown below:

```sway
{{#include ../../../../examples/storage_vec/src/main.sw:storage_vec_get}}
```

Note three details here. First, we use the index value of `2` to get the third element because vectors are indexed by number, starting at zero. Second, we get the third element by using the `get` method with the index passed as an argument, which gives us an `Option<T>`. Third, the ABI function calling `get` only requires the annotation `#[storage(read)]` as one might expect because `get` does not write to storage.

When the `get` method is passed an index that is outside the vector, it returns `None` without panicking. This is particularly useful if accessing an element beyond the range of the vector may happen occasionally under normal circumstances. Your code will then have logic to handle having either `Some(element)` or `None`. For example, the index could be coming as a contract method argument. If the argument passed is too large, the method `get` will return a `None` value, and the contract method may then decide to revert when that happens or return a meaningful error that tells the user how many items are in the current vector and give them another chance to pass a valid value.

## Iterating over the Values in a Vector

To access each element in a vector in turn, we would iterate through all of the valid indices using a `while` loop and the `len` method as shown below:

```sway
{{#include ../../../../examples/storage_vec/src/main.sw:storage_vec_iterate}}
```

Again, this is quite similar to iterating over the elements of a `Vec<T>` where we use the method `len` to return the length of the vector. We also call the method `unwrap` to extract the `Option` returned by `get`. We know that `unwrap` will not fail (i.e. will not cause a revert) because each index `i` passed to `get` is known to be smaller than the length of the vector.

## Using an Enum to store Multiple Types

Storage vectors, just like `Vec<T>`, can only store values that are the same type. Similarly to what we did for `Vec<T>` in the section [Using an Enum to store Multiple Types](./vec.md#using-an-enum-to-store-multiple-types), we can define an enum whose variants will hold the different value types, and all the enum variants will be considered the same type: that of the enum. This is shown below:

```sway
{{#include ../../../../examples/storage_vec/src/main.sw:storage_vec_multiple_types_enum}}
```

Then we can declare a storage vector in a `storage` block to hold that enum and so, ultimately, holds different types:

```sway
{{#include ../../../../examples/storage_vec/src/main.sw:storage_vec_multiple_types_decl}}
```

We can now push different enum variants to the storage vector as follows:

```sway
{{#include ../../../../examples/storage_vec/src/main.sw:storage_vec_multiple_types_fn}}
```

Now that we’ve discussed some of the most common ways to use storage vectors, be sure to review the API documentation for all the many useful methods defined on `StorageVec<T>` by the standard library. For now, these can be found in the [source code for `StorageVec<T>`](https://github.com/FuelLabs/sway/blob/master/sway-lib-std/src/storage.sw). For example, in addition to `push`, a `pop` method removes and returns the last element, a `remove` method removes and returns the element at some chosen index within the vector, an `insert` method inserts an element at some chosen index within the vector, etc.
