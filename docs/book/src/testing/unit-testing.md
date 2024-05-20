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

## Logs Inside Tests

<!-- This section should explain how log decoding works with Sway unit tests -->
<!-- unit_test_log::example::start -->
Forc has some capacity to help decode logs returned from the unit tests. You can use this feature to decode raw logs into a human readable format.

```sway
script;

fn main() {}

#[test]
fn test_fn() {
let a = 10;
    log(a);
    let b = 30;
    log(b);
    assert_eq(a, 10)
    assert_eq(b, 30)
}
```

The example shown above is logging two different variables, `a` and `b` and their values are `10` and `30`, respectively. Without log decoding printed log for this test with `forc test --logs` (`--logs` flag is required to see the logs for this example since the test is passing. Logs are silenced by default in passing tests, and can be enabled using the `--logs` flag.):

```console
Finished debug [unoptimized + fuel] target(s) in 5.23s
      Bytecode hash: 0x1cb1edc031691c5c08b50fd0f07b02431848ab81b325b72eb3fd233c67d6b548
   Running 1 test, filtered 0 tests
      test test_fn ... ok (38.875µs, 232 gas)
[{"LogData":{"data":"000000000000000a","digest":"8d85f8467240628a94819b26bee26e3a9b2804334c63482deacec8d64ab4e1e7","id":"0000000000000000000000000000000000000000000000000000000000000000","is":10368,"len":8,"pc":11032,"ptr":67107840,"ra":0,"rb":0}},{"LogData":{"data":"000000000000001e","digest":"48a97e421546f8d4cae1cf88c51a459a8c10a88442eed63643dd263cef880c1c","id":"0000000000000000000000000000000000000000000000000000000000000000","is":10368,"len":8,"pc":11516,"ptr":67106816,"ra":0,"rb":1}}]
```

This is not very easy to understand, it is possible to decode these logs with `--decode` flag, executing `forc test --logs --decode`:

```console
Finished debug [unoptimized + fuel] target(s) in 5.23s
      Bytecode hash: 0x1cb1edc031691c5c08b50fd0f07b02431848ab81b325b72eb3fd233c67d6b548
   Running 1 test, filtered 0 tests
      test test_fn ... ok (38.875µs, 232 gas)
Decoded log value: 10, log rb: 0
Decoded log value: 30, log rb: 1
```

As it can be seen, the values are human readable and easier to understand which makes debugging much more easier.

**Note**: This is an experimental feature and we are actively working on reporting variable names next to their values.
<!-- unit_test_log::example::end -->
