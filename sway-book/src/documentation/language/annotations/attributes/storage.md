# Storage

A storage attribute indicates the purity of a function i.e. whether it:

- reads from storage
- writes to storage
- reads from and writes to storage
- does not read or write (is pure)

When a function is pure the annotation is omitted otherwise the correct annotation must be placed above the function signature. 

More information about storage can be found in the [common storage operations](../../../operations/storage/index.md) section.

## Reading from Storage

When we read from storage we use the `read` keyword.

```sway
{{#include ../../../../code/language/annotations/src/main.sw:read}}
```

## Writing to Storage

When we write to storage we use the `write` keyword.

```sway
{{#include ../../../../code/language/annotations/src/main.sw:write}}
```

## Reading & Writing

When we read from and write to storage we use the `read` & `write` keywords.

```sway
{{#include ../../../../code/language/annotations/src/main.sw:read_write}}
```

