# A Forc Project

To initialize a new project with Forc, use `forc init`:

```sh
forc init hello_world
```

Here is the project that Forc has initialized:

```console
$ cd hello_world
$ tree .
├── Cargo.toml
├── Forc.toml
├── src
│   └── main.sw
└── tests
    └── harness.rs
```

`Forc.toml` is the _manifest file_ (similar to `Cargo.toml` for Cargo or `package.json` for Node), and defines project metadata such as the project name and dependencies.

```toml
{{#include ../../../examples/hello_world/Forc.toml}}
```

Here are the contents of the only Sway file in the project, and the main entry point, `src/main.sw`:

```sway
script;

fn main() {

}
```

The project is _script_, one of four different project types. For additional information on different project types, see [here](../sway-on-chain/index.md).

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

Compiled script "hello_world".
Bytecode size is 28 bytes.
```

To run this script, use `forc run` (note that `fuel-core` must be running for this to work):

```console
$ forc run
Bytecode size is 28 bytes.
[Return { id: ContractId([0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]), val: 0, pc: 488, is: 464 }]
```

Use `forc json-abi` to output the ABI of the contract. To write this to a `.json` file (which is necessary for running tests below), pipe it using something like:

```sh
forc json-abi > my-contract-abi.json
```

There is currently not a convention for where ABI files should be placed; one
common choice is loose in the root directory.
