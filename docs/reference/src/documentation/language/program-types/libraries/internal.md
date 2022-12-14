# Internal Libraries

A library is internal to a project if it is in the same source `src` directory as the other program files.

```bash
$ tree
.
├── Cargo.toml
├── Forc.toml
└── src
    ├── lib.sw
    └── my_library.sw
```

To be able to use our library `my_library.sw` in `lib.sw` there are two steps to take:

1. Bring our library into scope by using the `dep` keyword followed by the library name
2. Use the `use` keyword to selectively import various items from the library

```sway
{{#include ../../../../code/language/program-types/libraries/internal/my_library/src/lib.sw}}
```
