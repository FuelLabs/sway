# Library

A library is used to contain code that performs common operations in order to prevent code duplication. 

## Definition

Libraries are defined using the `library` keyword at the beginning of a file followed by a name so that they can be identified and imported.

```sway
{{#include ../../../../code/language/program-types/libraries/internal/my_library/src/my_library.sw:module}}
```

## Accessibility

Code defined inside a library, but more generally anywhere inside a Sway project, is considered to be `private` which means that it is inaccessible to other files unless explicitly exposed.

Code can be exposed through a two step process:

- Add a `pub` keyword at the start of some code
- Specify the [library](external.md) in the `Forc.toml` file as a dependency and then import the `pub` declaration

```sway
{{#include ../../../../code/language/program-types/libraries/internal/my_library/src/my_library.sw:library}}
```

The following structures can be marked as `pub`:

- Globally defined constants
- Structs
- Enums
- Functions

## Deployment

Libraries cannot be directly deployed to a blockchain however they can be deployed as part of a [contract](../contract.md).
