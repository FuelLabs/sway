# Unit Testing

<!-- This section should explain unit testing in Sway -->
<!-- unit_test:example:start -->
Forc provides built-in support for building and executing tests for a package.

Tests are written as free functions with the `#[test]` attribute.
<!-- unit_test:example:end -->

For example:

```sway
#[test]
fn test_meaning_of_life() {
    assert(6 * 7 == 42);
}
```

Each test function is ran as if it were the entry point for a
[script](../sway-program-types/scripts.md). Tests "pass" if they return
successfully, and "fail" if they revert or vice versa while [testing failure](#testing-failure).

If the project has failing tests `forc test` will exit with exit status `101`.

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

<!-- This section should explain support for failing unit tests in Sway -->
<!-- unit_test_fail:example:start -->
Forc supports testing failing cases for test functions declared with `#[test(should_revert)]`.
<!-- unit_test_fail:example:end -->

For example:

```sway
#[test(should_revert)]
fn test_meaning_of_life() {
    assert(6 * 6 == 42);
}
```

It is also possible to specify an expected revert code, like the following example.

```sway
#[test(should_revert = "18446744073709486084")]
fn test_meaning_of_life() {
    assert(6 * 6 == 42);
}
```

Tests with `#[test(should_revert)]` are considered to be passing if they are reverting.

## Calling Contracts

Unit tests can call contract functions an example for such calls can be seen below.

```sway
contract;

abi MyContract {
    fn test_function() -> bool;
}

impl MyContract for Contract {
    fn test_function() -> bool {
        true
    }
}
```

To test the `test_function()`, a unit test like the following can be written.

```sway
#[test]
fn test_success() {
    let caller = abi(MyContract, CONTRACT_ID);
    let result = caller.test_function {}();
    assert(result == true)
}
```

It is also possible to test failure with contract calls as well.

```sway
#[test(should_revert)]
fn test_fail() {
    let caller = abi(MyContract, CONTRACT_ID);
    let result = caller.test_function {}();
    assert(result == false)
}
```

<!-- This section should explain how the `CONTRACT_ID` variable works in Sway unit tests -->
<!-- contract_id:example:start -->
> **Note:** When running `forc test`, your contract will be built twice: first *without* unit tests in order to determine the contract's ID, then a second time *with* unit tests with the `CONTRACT_ID` provided to their namespace. This `CONTRACT_ID` can be used with the `abi` cast to enable contract calls within unit tests.
<!-- contract_id:example:end -->

Unit tests can call methods of external contracts if those contracts are added as contract dependencies, i.e. in the [`contract-dependencies`](../forc/manifest_reference.md#the-contract-dependencies-section) section of the manifest file. An example of such calls is shown below:

```sway
{{#include ../../../../examples/multi_contract_calls/caller/src/main.sw:multi_contract_calls}}
```

Example `Forc.toml` for contract above:

```toml
{{#include ../../../../examples/multi_contract_calls/caller/Forc.toml:multi_contract_call_toml}}
```

## Running Tests in Parallel or Serially

<!-- This section should explain how unit tests do not share storage -->
<!-- storage:example:start -->
By default, all unit tests in your project are run in parallel. Note that this does not lead to any data races in storage because each unit test has its own storage space that is not shared by any other unit test.
<!-- storage:example:end -->

By default, `forc test` will use all the available threads in your system. To request that a specific number of threads be used, the flag `--test-threads <val>` can be provided to `forc test`.

```console
forc test --test-threads 1
```
