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

If we take a look at each library then we see the following:

__my_library__

```sway
{{#include ../../../../code/language/program-types/libraries/external/my_library/src/lib.sw}}
```

__my_other_library__

```sway
{{#include ../../../../code/language/program-types/libraries/external/my_other_library/src/lib.sw}}
```

The code in `my_library` references `my_other_library` however there is one more step required in order to link the two libraries and that is to tell `my_library` where to find `my_other_library`.

This is done by listing `my_other_library` as a dependency in the `Forc.toml` file of `my_library` under the `[dependencies]` section.

```bash
{{#include ../../../../code/language/program-types/libraries/external/my_library/Forc.toml}}
```

> **Note:**
> Only libraries can be included in the manifest file. Other program types will error.
