# Testing with Rust

If you look again at the project structure when you create a [new Forc project](../introduction/forc_project.md) with `forc init`, you can see a directory called `tests/`:

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

Note that this is also a Rust package, hence the existence of a `Cargo.toml` (Rust manifest file) in the project root directory. The `Cargo.toml` in the root directory contains necessary Rust dependencies to enable you to write Rust-based tests using our [Rust SDK](https://github.com/FuelLabs/fuels-rs), (`fuels-rs`).

These tests can be run using `forc test` which will look for Rust tests under the `tests/` directory (created automatically with `forc init`).

For example, let's write tests against the following contract, written in Sway. This can be done in the pregenerated `src/main.sw` or in a new file in `src`. In the case of the latter, update the `entry` field in `Forc.toml` to point at the new contract.

```sway
{{#include ../../../examples/hello_world/src/main.sw}}
```

Our `tests/harness.rs` file could look like:

<!--TODO add test here once examples are tested-->
```rust,ignore
{{#include ../../../examples/hello_world/tests/harness.rs}}
```

Then, in the root of our project, running `forc test` will run the test above, compiling and deploying the contract to a local Fuel network, and calling the ABI methods against the contract deployed in there:

```console
$ forc test

running 1 test
test harness ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.64s
```
