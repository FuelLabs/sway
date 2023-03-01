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

Contracts can call other contract's functions if they are declared unedr [`contract-dependencies`](../forc/manifest_reference.md#the-contract-dependencies-section) table. An example for such calls can be seen below:

```sway
contract;

abi MyContract {
    fn test_true() -> bool;
}

impl MyContract for Contract {
    fn test_true() -> bool {
        true
    }
}

abi MyContract2 {
    fn test_false() -> bool;
}

#[test]
fn test_contract_multi_call() {
  let caller = abi(MyContract, CONTRACT_ID);

  let contract2_id = 0xad4770679dec457bd9c0875d5ea52d75ac735ef28c0187d0bf7ee1dff5b9cee3;
  let caller2 = abi(MyContract2, contract2_id);

  let should_be_true  = caller.test_true {}();
  let should_be_false = caller2.test_false {}();

  assert(should_be_true == true);
  assert(should_be_false == false);
}
```

> **Note:** In order for this example to work, the package hosting this contract must declare the package implementing the `MyContract2` ABI as a contract dependency. When running the `forc test` command, the IDs of any contracts that are dependencies of the currently tested contract will be displayed. 

> **Note:** When running `forc test`, your contract will be built twice: first *without* unit tests in order to determine the contract's ID, then a second time *with* unit tests with the `CONTRACT_ID` provided to their namespace. This `CONTRACT_ID` can be used with the `abi` cast to enable contract calls within unit tests.
