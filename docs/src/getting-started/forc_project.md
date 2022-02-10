# A Forc Project

To initialize a new project with Forc, use `forc init`:

```console
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

```console
forc json-abi > my-contract-abi.json
```

There is currently not a convention for where ABI files should be placed; one
common choice is loose in the root directory.

## Testing a Sway Project with Forc

If you look again at the project structure when you create a new Forc project, you can see a directory called `tests/`:

```plaintext
$ forc init my-fuel-project
$ cd my-fuel-project
$ tree .
├── Cargo.toml
├── Forc.toml
├── src
│   └── main.sw
└── tests
    └── harness.rs
```

Note that this is a Rust package, hence the existence of a `Cargo.toml` (Rust manifest file) in the project root directory. The `Cargo.toml` in the root directory contains necessary Rust dependencies to enable you to write Rust-based tests using our [Rust SDK](https://github.com/FuelLabs/fuels-rs) (`fuels-rs`).

These tests can be run using either `cargo test`, or `forc test` which will look for Rust tests under the `tests/` directory (created automatically with `forc init`).

For example, let's write tests against the following contract, written in Sway. This can be done in the pregenerated `src/main.sw` or in a new file in `src`. In the case of the latter, update the `entry` field in `Forc.toml` to point at the new contract.

```sway
{{#include ../../../examples/hello_world/src/main.sw}}
```

Our `tests/harness.rs` file could look like:

```rust
{{#include ../../../examples/hello_world/tests/harness.rs}}
```

Then, in the root of our project, running `forc test` or `cargo test` will run the test above, compiling and deploying the contract to a local Fuel network, and calling the ABI methods against the contract deployed in there:

```plaintext
$ forc test

running 1 test
test harness ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.64s
```

Instead of writing tests in Rust, tests can also be written in Typescript using our [Typescript SDK](https://github.com/FuelLabs/fuels-ts/).
