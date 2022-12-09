# Unit Testing

Forc provides built-in support for building and executing tests for a package.

Tests are written as free functions with the `#[test]` attribute. For example:

```sway
#[test]
fn test_meaning_of_life() {
    assert(6 * 7 == 42);
}
```

Each test function is ran as if it were the entry point for a
[script](../sway-program-types/scripts.md). Tests "pass" if they return
successfully, and "fail" if they revert or vice versa while [testing failure](#testing-failure).

## Building and Running Tests

We can build and execute all tests within a package with the following:

```console
forc test
```

The output should look similar to this:

```console
  Compiled library "core".
  Compiled library "std".
  Compiled library "lib_single_test".
  Bytecode size is 92 bytes.
   Running 1 tests
      test test_meaning_of_life ... ok (170.652µs)
   Result: OK. 1 passed. 0 failed. Finished in 1.564996ms.
```

Visit the [`forc test`](../forc/commands/forc_test.md) command reference to find
the options available for `forc test`.

## Testing Failure

Forc supports testing failing cases for test functions declared with `#[test(should_revert)]`. For example:

```sway
#[test(should_revert)]
fn test_meaning_of_life() {
    assert(6 * 6 == 42);
}
```

Tests with `#[test(should_revert)]` considered to be passing if they are reverting.

## Calling Contracts

***Coming Soon***

*Track progress on constract calls in tests
[here](https://github.com/FuelLabs/sway/issues/3262)*
