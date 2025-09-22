# Config Driven End To End Testing

Each of the tests in this suite are controlled by a TOML descriptor file which describes how the
test should be run and what result to expect if any.

## test.toml

To add a new test to the E2E suite place a `test.toml` file at the root of the test Forc package,
i.e., next to the `Forc.toml` file.  This file may contain a few basic fields.

## category

The `category` field is mandatory and must be one of the following strings:

- `"run"` - The test is compiled and run in a VM.
- `"run_on_node"` - The test is compiled and run on a local Fuel Core node.
- `"compile"` - The test is expected to succeed compiling, but isn't run in any way.
- `"unit_tests_pass"` - The test compiles and all unit tests pass successfully.
- `"fail"` - The test is expected to fail to compile.
- `"disabled"` - The test is disabled.

## expected_result

The `expected_result` field is mandatory for `"run"` and `"run_on_node"` tests.  It is a table with
two fields, `action` and `value`.

The `action` field describe what sort of result to expect:

- `"return"` - An integer value returned by success in the VM.
- `"return_data"` - An array of bytes returned by the VM.
- `"result"` - An integer word returned by the Fuel Core node.
- `"revert"` - An integer value returned by failure in the VM.

The `value` field is the actual expected value.  For `"return"`, `"result"` and `"revert"` actions
it must be an integer.

For `"return_data"` actions it must be an array of byte values, each an integer between 0 and 255.

## contracts

Tests in the `"run_on_node"` category will usually specify one or more contracts which must be
deployed to the node prior to deploying and running the test code.  These are specified with the
`contracts` field.

It must be an array of strings each containing only the path to the Forc project for the contract to
be compiled and deployed.  It is important that these paths remain relative to the
`test/src/e2e_vm_tests/test_programs` directory.

## validate_abi

Some tests also require their ABI is verified.  To indicate this the `validate_abi` field may be
specified, as a boolean value.

## supported_targets

Some tests are only compatible with some build targets. To indicate this the `supported_targets` field may be specified, as an array value.

## unsupported_profiles

Some tests can only work with some profiles (release / debug). By default, tests are tested with all profiles.
In case of incompability, the `unsupported_profiles` field may be specified, as an array value.

## expected_warnings

Some tests can have valid warnings. To allow them, specify `expected_warnings`, which takes an integer arguments.
If more than these many number of warnings are emitted, the test fails.

## FileCheck for 'fail' tests

The tests in the `fail` category _must_ employ verification using pattern matching via the [FileCheck](https://docs.rs/filecheck/latest/filecheck/)
crate.  The checker directives are specified in comments (lines beginning with `#`) in the `test.toml`
file.

Typically this is as simple as just adding a `# check: ...` line to the line specifying the full
error or warning message expected from compiling the test.  `FileCheck` also has other directives for
fancier pattern matching, as specified in the [FileCheck docs](https://docs.rs/filecheck/latest/filecheck/).

> **Note**
> The output from the compiler is colorized, usually to red or yellow, and this involves
printing ANSI escape sequences to the terminal.  These sequences can confuse `FileCheck` as it tries
to match patterns on 'word' boundaries.  The first word in an error message is most likely prefixed
with an escape sequence and can cause the check to fail.

To avoid this problem one may either not use the first word in the error message, or use the 'empty
string' pattern `$()` to direct the matcher as to where the pattern starts.

E.g, `# check: $()The imported symbol "S" shadows another symbol with the same name.`

## Examples

The following is a common example for tests in the `should_pass/language` directory.  The test
should be compiled, run on the VM, should expect a return value of 42 and should also validate the
ABI generation.

```toml
category = "run"
expected_result = { action = "return", value = 42 }
validate_abi = true
```

And this example is similar but expects return data.

> _Note:_ The return data expected is presented as a memory range because that's what the `RETD` opcode returns

```toml
category = "run"
expected_result = { action = "return_data", value = "0000000003ffffc400000000000000040000000000000003" }
validate_abi = true
```

The following tests a contract on a Fuel Core node.

```toml
category = "run_on_node"
expected_result = { action = "result", value = 11 }
contracts = ["should_pass/test_contracts/test_contract_a", "should_pass/test_contracts/test_contract_b"]
```

Tests which fail can have fairly elaborate checks.

```toml
category = "fail"

# check: // this asm block should return unit, i.e. nothing
# nextln: asm(r1: 5) {
# check: $()Mismatched types.
# nextln: $()expected: ()
# nextln: $()found:    u64.
# nextln: $()help: Implicit return must match up with block's type.
```

And there are already hundreds of existing tests with `test.toml` descriptors which may be consulted
when adding a new test.
