# Vectors on the Heap

The first collection type we’ll look at is `Vec<T>`, also known as a vector. Vectors allow you to store more than one value in a single data structure that puts all the values next to each other in memory. Vectors can only store values of the same type. They are useful when you have a list of items, such as the lines of text in a file or the prices of items in a shopping cart.

In order to use `Vec<T>`, you must first import `Vec` as follows:

```sway
{{#include ../../../examples/vec/src/main.sw:vec_import}}
```

## Creating a New Vector

To create a new empty vector, we call the `Vec::new` function, as shown below:

```sway
{{#include ../../../examples/vec/src/main.sw:vec_new}}
```

Note that we added a type annotation here. Because we aren’t inserting any values into this vector, the Sway compiler doesn’t know what kind of elements we intend to store. Vectors are implemented using generics which means that the `Vec<T>` type provided by the standard library can hold any type. When we create a vector to hold a specific type, we can specify the type within angle brackets. In the example above, we’ve told the Sway compiler that the `Vec<T>` in `v` will hold elements of the `u64` type.

## Updating a Vector

To create a vector and then add elements to it, we can use the `push` method, as shown below:

```sway
{{#include ../../../examples/vec/src/main.sw:vec_push}}
```

As with any variable, if we want to be able to change its value, we need to make it mutable using the `mut` keyword, as discussed in the section [Declaring a Variable](../basics/variables.md#declaring-a-variable). The numbers we place inside are all of type `u64`, and the Sway compiler infers this from the data, so we don’t need the `Vec<u64>` annotation.

## Reading Elements of Vectors

To read a value stored in a vector at a particular index, you can use the `get` method as shown below:

```sway
{{#include ../../../examples/vec/src/main.sw:vec_get}}
```

Note two details here. First, we use the index value of `2` to get the third element because vectors are indexed by number, starting at zero. Second, we get the third element by using the `get` method with the index passed as an argument, which gives us an `Option<T>`.

When the `get` method is passed an index that is outside the vector, it returns `None` without panicking. This is particularly useful if accessing an element beyond the range of the vector may happen occasionally under normal circumstances. Your code will then have logic to handle having either `Some(element)` or `None`. For example, the index could be coming as a contract method argument. If the argument passed is too large, the method `get` will return a `None` value, and the contract method may then decide to revert when that happens or return a meaningful error that tells the user how many items are in the current vector and give them another chance to pass a valid value.

## Iterating over the Values in a Vector

To access each element in a vector in turn, we would iterate through all of the valid indices using a `while` loop and the `len` method as shown below:

```sway
{{#include ../../../examples/vec/src/main.sw:vec_iterate}}
```

Note two details here. First, we use the method `len` which returns the length of the vector. Second, we call the method `unwrap` to extract the `Option` returned by `get`. We know that `unwrap` will not fail (i.e. will not cause a revert) because each index `i` passed to `get` is known to be smaller than the length of the vector.

## Using an Enum to store Multiple Types

Vectors can only store values that are the same type. This can be inconvenient; there are definitely use cases for needing to store a list of items of different types. Fortunately, the variants of an enum are defined under the same enum type, so when we need one type to represent elements of different types, we can define and use an enum!

For example, say we want to get values from a row in a table in which some of the columns in the row contain integers, some `b256` values, and some Booleans. We can define an enum whose variants will hold the different value types, and all the enum variants will be considered the same type: that of the enum. Then we can create a vector to hold that enum and so, ultimately, holds different types. We’ve demonstrated this below:

```sway
{{#include ../../../examples/vec/src/main.sw:vec_multiple_data_types}}
```

Now that we’ve discussed some of the most common ways to use vectors, be sure to review the API documentation for all the many useful methods defined on `Vec<T>` by the standard library. For now, these can be found in the [source code for `Vec<T>`](https://github.com/FuelLabs/sway/blob/master/sway-lib-std/src/vec.sw). For example, in addition to `push`, a `pop` method removes and returns the last element, a `remove` method removes and returns the element at some chosen index within the vector, an `insert` method inserts an element at some chosen index within the vector, etc.
