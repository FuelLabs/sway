# IR Generation and ASM Generation Tests

These tests are for validating specific aspects of IR and ASM generation as opposed to full end-to-end testing.

## How to author new tests

To create a new test write a small snippet of Sway to target the specific IR or ASM construct. Placing this file under `ir_generation/tests` will automatically add it to the suite.

The [FileCheck](https://docs.rs/filecheck/latest/filecheck/) crate is used to verify the output from the compiler.  If no `FileCheck` directives are found then the harness will print to screen the IR or ASM output which will be tested against, and the required directives can be based on that text.

> **Note**
> See the existing tests in the `ir_generation/tests` for examples.

## Built in `regex` directives

Some commonly used `FileCheck` `regex` directives are provided by the harness for use in matching particular IR and ASM tokens:

* `VAL` - matches `v\d+v\d+` which is the syntax for IR values.
* `ID` - matches `[_[:alpha:]][_0-9[:alpha:]]*` which is an identifier such as an IR block or function name.
* `MD` - matches `!\\d+` which is an IR metadata index.

These built in directives are already used extensively in the suite.

## Delimiting markers

Both checks against IR and ASM may be provided in the same Sway test source file.  IR checks are mandatory and ASM checks are optional.

To delimit between checks against IR and those against ASM the source file may be split into sections using delimiting marker text.

* `::check-ir::` marks the beginning of the IR checks.
* `::check-ir-optimized::` marks the beginning of the optimized IR checks.
* `::check-asm::` marks the beginning of the ASM checks.

  Optimized IR checker can be configured with `pass: <PASSNAME or o1>`. When
  `o1` is chosen, all the configured passes are chosen automatically.
  ```
  ::check-ir-optimized::
  pass: o1
  ```

The sections may go in either order.  If there are no markers then it is assumed that all checks are for IR.
