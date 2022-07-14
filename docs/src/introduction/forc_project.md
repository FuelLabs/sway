# A Forc Project

To initialize a new project with Forc, use `forc new`:

```sh
forc new my-fuel-project
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

We now compile our project with `forc build`, passing the flag `--print-finalized-asm` to view the generated assembly:

```console
$ forc build --print-finalized-asm
.program:
ji   i4
noop
DATA_SECTION_OFFSET[0..32]
DATA_SECTION_OFFSET[32..64]
lw   $ds $is 1
add  $$ds $$ds $is
lw   $r1 $fp i73              ; load input function selector
lw   $r0 data_1               ; load fn selector for comparison
eq   $r0 $r1 $r0              ; function selector comparison
jnzi $r0 i11                  ; jump to selected function
rvrt $zero                    ; revert if no selectors matched
lw   $r0 data_0               ; literal instantiation
ret  $r0
.data:
data_0 .bool 0x01
data_1 .u32 0x2151bd4b

  Compiled contract "my-fuel-project".
  Bytecode size is 68 bytes.
```

To test this contract, use `forc test`:

```console
$ forc test
running 1 test
test can_get_contract_id ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.22s
```

The `forc test` command tests the contract using the Rust SDK test harness that lives under `tests/`. The default test harness `harness.rs` contains boilerplate code to get you started but doesn't actually call any contract methods. For additional information on testing contracts using the Rust SDK, refer to the [Testing with Rust](../testing/testing-with-rust.md) section.
