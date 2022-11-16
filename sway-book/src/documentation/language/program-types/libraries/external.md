# External Libraries

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

## Libraries

### my_other_library

`my_other_library` has a function `quix()` which can be imported into `my_library` because it uses the `pub` keyword.

```sway
{{#include ../../../../code/language/program-types/libraries/external/my_other_library/src/lib.sw}}
```

### my_library

To be able to use `quix()` inside `my_library` there are two steps to take.

#### Add to Dependencies

Add `my_other_library` as a dependency under the `[dependencies]` section of the `my_library` `Forc.toml` file.

```bash
{{#include ../../../../code/language/program-types/libraries/external/my_library/Forc.toml}}
```

#### Import

Use the `use` keyword to selectively import our code from `my_other_library`

```sway
{{#include ../../../../code/language/program-types/libraries/external/my_library/src/lib.sw}}
```
