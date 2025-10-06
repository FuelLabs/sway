# Snapshot tests

There are two ways to run snapshot tests:

```
> cargo r -p test --release
> cargo r -p test --release -- -k snapshot
```

When the snapshot flag is enabled (the default) the test harness will search for `snapshot.toml` files. For every `toml` file found a new snapshot test will run. If the `toml` file is empty, it will be interpreted as simply being:

```toml
# this is how the test harness understand when it sees an empty snapshot.toml
cmds = [
    \"forc build --path {root}\"
]
```

When the test harness runs a snapshot test, it will iterate the `cmds` array of each file, run each command, and append everything into the snapshot.

So the snapshot of the above file would be something like

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

1 - First it does not show complete file paths. All paths are relative to the swat repo root.
2 - Test harness also remove all printing of times.
3 - It also removes all ANSI codes for coloring and others.

## Commands

To make snapshot tests more versalite there are a lot of options of what one can use inside `cmds`:

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

The example above will only show asm lines that contains the `ecal` instruction.

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

Some commands manipulate files. These commands have an "undo" list that will restore the file original content when they a finished.
So it is guaranteed that if the test harness finishes gracefully, `main.sw` will have its original value.

For security reasons, these commands can ONLY manipulate files under its project folder.

1 - `replace`. Replace a simple string for another.

```toml
cmds = [
    "replace src/main.sw \"fn cost_of_in\" \"fn isolated_cost_of_in\"",
]
```

## Variables

1 - `root`. Is the folder of the project being compiled.
2 - `name`. Is the name of the specific project being compiled.

## Blocks

Blocks are blocks of code that live inside two comments of the form below:

```rust
/* START BOOL */
#[test]
fn cost_of_in_bool() {
    let _ = abi(MyContract, CONTRACT_ID).in_bool(false);
}
/* END BOOL */
```

These blocks can be manipulade from inside the snapshot.toml and allowing multiples tests to use the same project.
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
