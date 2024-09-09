# Initialization

Storage is declared through the use of the `storage` keyword.

Inside the `storage` block each variable is named, associated with a type and a default value.

```sway
{{#include ../../../code/operations/storage/empty_storage_init/src/main.sw:initialization}}
```

## Example

In the following example we will take a look at two ways of storing a struct.

- Explicitly declaring the values in the `storage` block
- Encapsulating the values in an [associated function](../../language/functions/index.md)

We'll begin by defining the `Owner` & `Role` data structures and implement a `default` [constructor](../../language/functions/associated-function.md#constructors) on the `Owner`.

```sway
{{#include ../../../code/operations/storage/storage_init/src/main.sw:data_structures}}
```

Now that we have our data structures we'll keep track of how many `current_owners` we have and declare the owner in the two aforementioned styles.

```sway
{{#include ../../../code/operations/storage/storage_init/src/main.sw:initialization}}
```

An explicit declaration is likely to be sufficient for most types. However, it may be preferable to encapsulate the initialization of complex types within a [constructor](../../language/functions/associated-function.md#constructors) in order to keep the code concise.

Note that the constructors used in `storage` blocks must evaluate to a constant during compilation.
