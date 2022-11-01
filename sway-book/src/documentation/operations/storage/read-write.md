# Reading & Writing

When dealing with storage we have two options, we can either read from or write to storage. In both cases we must use a [storage annotation](../../language/annotations/attributes/storage.md) to indicate the purity of the function.

When referencing a variable in storage we must explicitly indicate that the variable comes from storage and not a local scope.

This is done via the syntax `storage.variable_name` e.g. `storage.counter`.

```sway
{{#include ../../../code/operations/storage/reading_writing_to_storage/src/main.sw:declaration}}
```

## Reading from Storage

When dealing with a [built-in](../../language/built-ins/index.md) type we can retrieve the variable without the use of any special methods.

```sway
{{#include ../../../code/operations/storage/reading_writing_to_storage/src/main.sw:read}}
```

## Writing to Storage

When dealing with a [built-in](../../language/built-ins/index.md) type we can update the variable without the use of any special methods.

```sway
{{#include ../../../code/operations/storage/reading_writing_to_storage/src/main.sw:write}}
```

## Reading & Writing

We can read and write to storage by using both keywords in the attribute.

```sway
{{#include ../../../code/operations/storage/reading_writing_to_storage/src/main.sw:read_write}}
```
