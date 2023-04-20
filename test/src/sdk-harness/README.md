# Building & running the sway-lib-std tests

## Building test projects

First, ensure we have the current version of `forc` installed.

```sh
cd sway
cargo install --path ./forc
```

In the `sway/test/src/sdk-harness`, compile all the Sway programs in the workspace:

```sh
cd sway/test/src/sdk-harness
forc build
```

This will build all contracts and scripts under the `sway/test/src/sdk-harness` directory.
After a successful build of all the projects:

```sh
cargo test
```

For more on the usage of Forc for testing, see: <https://fuellabs.github.io/sway/latest/forc/commands/forc_test.html>
