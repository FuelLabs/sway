
# forc-test
Run Rust-based tests on current project. As of now, `forc test` is a simple wrapper on `cargo test`;
`forc init` also creates a rust package under your project, named `tests`. You can opt to either run
these Rust tests by using `forc test` or going inside the package and using `cargo test`


## USAGE:
forc test [TEST_NAME]


## ARGS:

<_TEST_NAME_>

   If specified, only run tests containing this string in their names


## OPTIONS:

`-h`, `--help`, 

Print help information
