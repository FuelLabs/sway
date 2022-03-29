# Building & running the sway-lib-std tests

As we currently don't have a CI job which runs these automatically, make sure tests are all passing before requesting a PR review or merging to master.

## Build

In the root of the `sway` repo is a bash build script. To run it:

```sh
cd sway
./build.sh
```

This will build all contracts and scripts under the `sway/sway-lib-std/tests/` directory, and then run `cargo test` which runs the entire `sway-lib-std` test suite.

If you want more fine-grained control over running tests, you may remove/comment out the `cargo test` command near the end of `build.sh`.

Then, after a sucessfull build of all the projects:

```sh
cd sway-lib-std/tests
```

- To run all tests single threaded:

```sh
cargo test -- cargo test --  --test-threads=1
```

- To capture output (ie: logs from println!) even for passing tests:

```sh
cargo test --  --test-threads=1 --no-capture
```

- To run a subset of tests, use the filter option. eg:

```sh
cargo test -- token_ops --test-threads=1 --no-capture
```

will run only the "token_ops" tests
