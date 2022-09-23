# Libraries

Libraries are defined using the `library` keyword at the beginning of a file, followed by a name so that they can be imported.

```sway
{{#include ../../code/libraries/internal/my_library/src/my_library.sw:1}}
```

All of the code inside the library is private by default therefore if the library is meant to expose some functionality then a `pub` keyword should be used in order to expose it.

```sway
{{#include ../../code/libraries/internal/my_library/src/my_library.sw}}
```

## Including a library in a project

There are two ways to include a library in a project.

### Internal Libraries

A library is internal to a project if it is in the same source `src` directory as the other program files.

```bash
$ tree
.
├── Cargo.toml
├── Forc.toml
├── src
│   ├── lib.sw
│   └── my_library.sw
└── tests
    └── harness.rs
```

To be able to use our library `my_library.sw` in `lib.sw` there are two steps to take:

1. Bring our library into scope by using the `dep` keyword followed by the library name
2. Use the `use` keyword to selectively import our code from the library

```sway
{{#include ../../code/libraries/internal/my_library/src/lib.sw}}
```

### External Libraries

An external library is a library that is outside of the `src` directory (most likely in an entirely different project).

```bash
$ tree
.
├── my_library
│   ├── Cargo.toml
│   ├── Forc.toml
│   ├── src
│   │   └── lib.sw
│   └── tests
│       └── harness.rs
└── my_other_library
    ├── Cargo.toml
    ├── Forc.toml
    ├── src
    │   └── lib.sw
    └── tests
        └── harness.rs
```

If we take a look at each library then we have the following:

__my_library__

```sway
{{#include ../../code/libraries/external/my_library/src/lib.sw}}
```

__my_other_library__

```bash
$ cat my_other_library/src/lib.sw
library my_other_library;

pub fn quix() {}
```

The code in `my_library` seems to use the code from `my_other_library` however there is one more step required to let `my_library` know about the path where it can find `my_other_library`.

This is done by listing `my_other_library` as a dependency in the `Forc.toml` file of `my_library` under the `[dependencies]` section.

```bash
{{#include ../../code/libraries/external/my_library/Forc.toml}}
```

> **Note:**
> Only libraries can be included in the manifest file. Other program types will error.
