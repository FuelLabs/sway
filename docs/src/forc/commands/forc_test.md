# forc-test
Run Rust-based tests on current project. As of now, `forc test` is a simple wrapper on `cargo test`;
`forc init` also creates a rust package under your project, named `tests`. You can opt to either run
these Rust tests by using `forc test` or going inside the package and using `cargo test`


## USAGE:
forc test [OPTIONS] [TEST_NAME] [-- <CARGO_TEST_ARGS>...]


## ARGS:

<_TEST_NAME_>
If specified, only run tests containing this string in their names


<_CARGO_TEST_ARGS_>

..
All trailing arguments following `--` are collected within this argument.

E.g. Given the following:

`forc test -- foo bar baz`

The arguments `foo`, `bar` and `baz` are forwarded on to `cargo test` like so:

`cargo test -- foo bar baz`


## OPTIONS:

`--cargo-test-opts` <_CARGO_TEST_OPTS_>


Options passed through to the `cargo test` invocation.

E.g. Given the following:

`forc test --cargo-test-opts="--color always"`

The `--color always` option is forwarded to `cargo test` like so:

`cargo test --color always`


`-h`, `--help` 


Print help information

## EXAMPLES:

You can write tests in Rust using our [Rust SDK](https://github.com/FuelLabs/fuels-rs). These tests can be run using `forc test`, which will look for Rust tests under the `tests/` directory (which is created automatically with `forc init`).

You can find an example under the [Testing with Rust](../../testing/testing-with-rust.md) section.