# A Forc Project

To initialize a new project with Forc, use `forc new`:

```sh
forc new my-fuel-project
```

Here is the project that Forc has initialized:

<!-- This section should show the tree for a new forc project -->
<!-- tree:example:start -->
```console
$ cd my-fuel-project
$ tree .
├── Forc.toml
└── src
    └── main.sw
```
<!-- tree:example:end -->

<!-- This section should explain the `Forc.toml` file -->
<!-- forc_toml:example:start -->
`Forc.toml` is the _manifest file_ (similar to `Cargo.toml` for Cargo or `package.json` for Node), and defines project metadata such as the project name and dependencies.
<!-- forc_toml:example:end -->

For additional information on dependency management, see: [here](../forc/dependencies.md).

```toml
[project]
authors = ["User"]
entry = "main.sw"
license = "Apache-2.0"
name = "my-fuel-project"

[dependencies]
```

Here are the contents of the only Sway file in the project, and the main entry point, `src/main.sw`:

```sway
contract;

abi MyContract {
    fn test_function() -> bool;
}

impl MyContract for Contract {
    fn test_function() -> bool {
        true
    }
}
```

The project is a _contract_, one of four different project types. For additional information on different project types, see [here](../sway-program-types/index.md).

We now compile our project with `forc build`, passing the flag `--asm final` to view the generated assembly:

```console
$ forc build --asm final
...
.program:
ji   i4
noop
DATA_SECTION_OFFSET[0..32]
DATA_SECTION_OFFSET[32..64]
lw   $ds $is 1
add  $$ds $$ds $is
lw   $r0 $fp i73              ; load input function selector
lw   $r1 data_0               ; load fn selector for comparison
eq   $r2 $r0 $r1              ; function selector comparison
jnzi $r2 i12                  ; jump to selected function
movi $$tmp i123               ; special code for mismatched selector
rvrt $$tmp                    ; revert if no selectors matched
ret  $one
.data:
data_0 .word 559005003

  Compiled contract "my-fuel-project".
  Bytecode size is 60 bytes.
```
