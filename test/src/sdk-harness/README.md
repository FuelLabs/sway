# Building & running the sway-lib-std tests

## Building test projects

First, ensure we have the current version of `forc` installed.

```sh
$ cd sway
$ cargo install --path ./forc
```

In the root of the `sway-lib-std` is a bash build script. To run it:

```sh
$ cd sway-lib-std
$ ./build.sh
```

This will build all contracts and scripts under the `sway/sway-lib-std/tests/` directory.
After a sucessfull build of all the projects:

```sh
$ cd tests
```

## To run all tests single threaded

```sh
$ cargo test --  --test-threads=1
```

## To capture output (ie: logs from println!) even for passing tests

```sh
$ cargo test --  --test-threads=1 --nocapture
```

## To run a subset of tests, use the filter option

```sh
$ cargo test -- token_ops --test-threads=1 --no-capture
```

The above example will run only the "token_ops" tests
