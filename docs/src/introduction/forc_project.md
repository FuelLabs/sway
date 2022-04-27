# A Forc Project

To initialize a new project with Forc, use `forc init`:

```sh
forc init my-fuel-project
```

Here is the project that Forc has initialized:

```console
$ cd my-fuel-project
$ tree .
├── Cargo.toml
├── Forc.toml
├── src
│   └── main.sw
└── tests
    └── harness.rs
```

`Forc.toml` is the _manifest file_ (similar to `Cargo.toml` for Cargo or `package.json` for Node), and defines project metadata such as the project name and dependencies.

For additional information on dependency management, see: [here](../forc/dependencies.md).

```toml
{{#include ../../../examples/counter/Forc.toml}}
```

Here are the contents of the only Sway file in the project, and the main entry point, `src/main.sw`:

```sway
script;

fn main() {

}
```

The project is a _script_, one of four different project types. For additional information on different project types, see [here](../sway-program-types/index.md).

We now compile our project with `forc build`, passing the flag `--print-finalized-asm` to view the generated assembly:

```console
$ forc build --print-finalized-asm
.program:
ji   i4
noop
DATA_SECTION_OFFSET[0..32]
DATA_SECTION_OFFSET[32..64]
lw   $ds $is 1
add  $ds $ds $is
ret  $zero                    ; main fn returns unit value
.data:

Compiled script "my-fuel-project".
Bytecode size is 28 bytes.
```

To run this script, use `forc run` (note that `fuel-core` must be running for this to work):

```console
$ forc run
Bytecode size is 28 bytes.
[Return { id: ContractId([0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]), val: 0, pc: 488, is: 464 }]
```
