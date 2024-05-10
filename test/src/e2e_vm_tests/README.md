# Writing end-to-end VM tests

In order to minimize compilation time of individual tests, strive to reduce dependencies in tests.

To achieve that, follow these guidelines:
- Use `implicit-std = false` if dependency on `core` is not needed. This is often possible when testing `should_pass/language` features.
- Do not use `std` just to conveniently get an arbitrary type or trait. E.g., if a test requires an arbitrary type or trait, go with `struct Dummy {}` or `trait Trait {}` instead of importing `Option` or `Hash`.
- If `std` functionality is needed, import the minimal [reduced `std` library](reduced_std_libs/README.md) that provides the functionality.
- Import the full `std` only if the provided [reduced `std` libraries](reduced_std_libs/README.md) do not provide required types.

# Running end-to-end VM tests

This document assumes you have `fuel-core` running on the default port.

## Running all tests

In a second terminal, `cd` into the `sway` repo and run:

```sh
cargo run --bin=test
```

After the test suite runs, you should see:

```console
Tests passed.
_n_ tests run (0 skipped)
```

By default, `cargo` uses the debug build mode which might result in slow
execution. To speed testing up, you might want to use the release build mode:

```sh
cargo run --release --bin=test
```

This may speed the tests up by an order of magnitude.

## Running specific tests

From the `sway` directory run:

```sh
cargo run --bin=test -- specific_tests_pattern
```

The `test` crate supports filtering out tests with a regex, i.e.
`specific_tests_pattern` above.

For instance, the following command

```sh
cargo run --bin=test -- abi_impl
```

would only run tests with the `abi_impl` substring in their names and might
produce output similar to the following:

```console
    Finished dev [unoptimized + debuginfo] target(s) in 0.66s
     Running `target/debug/test abi_impl`
 Compiling should_fail/abi_impl_purity_mismatch
 Compiling should_fail/abi_impl_purity_mismatch
 Compiling should_fail/too_many_abi_impl_methods
 Compiling should_fail/too_many_abi_impl_methods
 Compiling should_fail/abi_impl_pub_fn
 Compiling should_fail/abi_impl_pub_fn
 Compiling should_fail/abi_impl_arity_mismatch
 Compiling should_fail/abi_impl_arity_mismatch
_________________________________
Tests passed.
Ran 4 out of 322 E2E tests (0 disabled).
No IR generation tests were run. Regex filter "abi_impl" filtered out all 48 tests.
```

Note that if you `cd` into the `sway/test` directory, you can just say `cargo run [pattern]`.

## Getting more information while running tests

To print out the warnings and errors run

```shell
SWAY_TEST_VERBOSE=true cargo run [pattern]
```

from the `sway/test` directory.
