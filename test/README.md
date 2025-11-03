# Using testing infrastructure

## Snapshot tests

There are two ways to run snapshot tests:

```
> cargo r -p test --release
> cargo r -p test --release -- -k snapshot
```

When the snapshot flag is enabled (the default) the test harness will search for `snapshot.toml` files. For every `toml` file found a new snapshot test will run. If the `toml` file is empty, it will be interpreted as simply being:

```toml
# this is how the test harness understands an empty `snapshot.toml
cmds = [
    \"forc build --path {root}\"
]
```

When the test harness runs a snapshot test, it will iterate the `cmds` array of each file, run each command, and append everything into the snapshot.

So the snapshot of the above file would be something like:

```
> forc test --path test/src/e2e_vm_tests/test_programs/should_pass/test_contracts/const_of_contract_call
exit status: 0
output:
    Building test/src/e2e_vm_tests/test_programs/should_pass/test_contracts/const_of_contract_call
   Compiling library std (test/src/e2e_vm_tests/reduced_std_libs/sway-lib-std-core)
   Compiling contract const_of_contract_call (test/src/e2e_vm_tests/test_programs/should_pass/test_contracts/const_of_contract_call)
    Finished release [optimized + fuel] target(s) [1.88 KB] in ???
```

To make snapshot "environment free", the test harness changes `forc` output a little bit.

1 - First it does not show complete file paths. All paths are relative to the Sway repo root.
2 - Test harness also removes all printing of times.
3 - It also removes all ANSI codes for coloring and others.

### Commands

To make snapshot tests more versatile there are a lot of options of what one can use inside `cmds`:

1 - `forc`; Any native forc command is available;
2 - Forc plugins; Currently only `forc doc` is available;
3 - `echo`; One can use echo to write a message. Usage is:

```toml
cmds = [
    "echo Explain something here.",
    "forc build --path {root}",
]
```

4 - `sub`; Sub will iterate all lines of the previous command and filter in only those that contain its argument.

```toml
cmds = [
    "forc build --path {root} --asm final | sub ecal"
]
```

The example above will only show asm lines that contain the `ecal` instruction.

5 - `regex`; Regex is very similar to `sub`, but allows a regex to be written.

```toml
cmds = [
    "forc build --path {root} --ir initial | regex '        (v0 = call call|v0 = const|v1 = const|revert)'"
]
```

6 - `filter-fn`. This command only shows IR, ASM for a specific function. It needs that previous command return a complete IR of a program. 

```toml
cmds = [
    "forc build --path {root} --ir final --asm final | filter-fn {name} transmute_by_reference_7",
]
```

In the example above, the snapshot will only contain IR and asm from the function "transmute_by_reference_7"

Some commands manipulate files. These commands have an "undo" list that will restore the file original content when they are finished.
So it is guaranteed that if the test harness finishes gracefully, manipulated files will have their original values.

For security reasons, these commands can ONLY manipulate files under its project folder.

1 - `replace`. Replace a simple string for another.

```toml
cmds = [
    "replace src/main.sw \"fn cost_of_in\" \"fn isolated_cost_of_in\"",
]
```

### Variables

1 - `root`. Is the folder of the project being compiled.
2 - `name`. Is the name of the specific project being compiled.

### Blocks

Blocks are blocks of code that live inside two comments of the form below:

```sway
/* START BOOL */
#[test]
fn cost_of_in_bool() {
    let _ = abi(MyContract, CONTRACT_ID).in_bool(false);
}
/* END BOOL */
```

These blocks can be manipulated from inside the `snapshot.toml` and allowing multiples tests to use the same project.
To manipulate these blocks one can:

1 - Use the `repeat-for-each-block`. 

```toml
cmds = [
    { repeat = "for-each-block", cmds = [
        "forc test --path {root} --release --experimental const_generics"
    ] }
]
```

In the example above, the test harness will collect all "blocks" in the project being compiled, and will run the `cmds` inside the inner table for each block, removing all others. So for example:

```rust
/* START BOOL */
#[test]
fn cost_of_in_bool() {
    let _ = abi(MyContract, CONTRACT_ID).in_bool(false);
}
/* END BOOL */

/* START U8 */
#[test]
fn cost_of_in_u8() {
    let _ = abi(MyContract, CONTRACT_ID).in_u8(0);
}
/* END U8 */
```

In the example above, the `repeat = "for-each-block"` means that its `cmds` list will be run twice. First removing the block `U8`; and after, it will restore the original file contents and remove "BOOL".

## Collecting and analyzing performance data (bytecode sizes and gas usages)

### Collecting performance data from end-to-end tests

E2E test runner has `--perf` flag that instructs it to output performance data, gas usages and bytecode sizes, for tests of the following categories:
- compile
- run
- unit_tests_pass

If we are interested only in generating performance data, and not in running all tests, we can in addition use `--perf-only` flag, which will filter the tests to only those of the above three categories:

```console
cargo r -r -p test -- --kind e2e --release --perf-only --perf
```

Collected gas usages and bytecode sizes are written to .gitignored files named `<timestamp>-<test kind>-<perf data kind>-<build profile>-<branch>.csv`, located in the `./test/perf_out` folder. E.g.: `./test/perf_out/1020165605-e2e-bytecode-sizes-release-master.csv`.

### Using `performance` `just` recipes

The following `performance` `just` recipes support easy creation and comparison of performance data stored in such CSV files:

```console
[performance]
perf-e2e filter=''                    # collect gas usages and bytecode sizes from E2E tests [alias: pe2e]
perf-in-lang filter=''                # collect gas usages from in-language tests [alias: pil]
perf-all filter=''                    # collect gas usages and bytecode sizes from all tests (E2E and in-language) [alias: pa]
perf-diff before after format='md'    # generate performance diff between two CSV files [alias: pd]
perf-diff-stats diff_file             # generate performance statistics summary from a `perf-diff` output CSV file [alias: pds]
perf-diff-latest format='md'          # generate performance diffs between the latest two CSV files per testing category [alias: pdl]
perf-snapshot-historical path open='' # collect historic gas usages from a snapshot test that has a `forc test` output [alias: psh]
perf-list                             # list all performance files (*gas-usages-*.* and *bytecode-sizes-*.*) [alias: pl]
perf-remove                           # remove all performance files (*gas-usages-*.* and *bytecode-sizes-*.*) [alias: pr]
```

Those `just` recipes should be executed from the root of the Sway repository. Every recipe prints the files it reads from or creates.

E.g., to create performance data CSV files, you can use `perf-e2e`, `perf-in-lang`, or `perf-all`, as explained in recipes' comments above:

| Command | Meaning |
| ------- | ------- |
| just perf-pe2e | Runs `--perf-only` e2e tests and writes output to files in `./test/perf_out`. |
| just perf-pe2e test_name | Runs `--perf-only` e2e tests with the `test_name` regex and writes output to files in `./test/perf_out`. |

To explicitly compare two CSV files `perf-diff` recipe can be used. The files that are compared must contain exactly the same test names in the same order. This will mostly be the case, but in cases when we are comparing performance data coming from two branches with slightly different tests, the listed tests can differ. **I that case, you need to manually adjust the tests, e.g., deleting those non-existing in both branches, before running `just perf-diff`.** The `perf-diff` recipe will print the mismatching tests and abort comparison if the tests are not the same in both files.

Results of `perf-diff` comparisons are also written to the `./test/perf_out` folder and named as `<timestamp>-diff-<perf data kind>-<timestamp before>-vs-<timestamp after>.{csv|md}`. E.g., `1020170324-diff-e2e-gas-usages-1020165605-vs-1020165756.md`.

However, in the most common case, we just want to compare the last two generated CSV files per performance category. The easiest way to achieve this is to call the `perf-diff-latest` recipe: `just perf-diff-latest`.

The most usual workflow of collecting and comparing performance data between the `master` and a feature branch can, thus, be achieved by running just these three `just` recipes (using recipes' aliases instead of full names):

```console
just pa   // `just perf-all`. First run on `master` to get the baseline.
          // Switch to the feature branch.
just pa   // `just perf-all`. Run on the feature branch to get the improved performance data.
just pdl  // `just perf-diff-latest`. Run to get the diff of those last two collected performance data sets.
```

Generated `*-bytecode-sizes-*` and `*-gas-usages-*` files (CSV, MD, HTML) can be seen using `just pl` and removed using `just pr`.
