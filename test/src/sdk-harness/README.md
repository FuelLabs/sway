# Building & running the sway-lib-std tests

## Building test projects

Compile all the Sway programs in the workspace as follows:

```sh
# from project root
cargo run --bin=forc build --path test/src/sdk-harness
```

This will build all contracts and scripts under the `sway/test/src/sdk-harness` directory.
After a successful build of all the projects:

```sh
cargo test
```

For more on the usage of Forc for testing, see: <https://fuellabs.github.io/sway/latest/forc/commands/forc_test.html>
