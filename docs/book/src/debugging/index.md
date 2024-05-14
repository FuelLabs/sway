# Debugging

Forc provides tools for debugging both live transactions as well as Sway unit tests.
Debugging can be done via CLI or using the VSCode IDE.

**Unit testing** refers to "in-language" test functions annotated with `#[test]`. Line-by-line
debugging is available within the VSCode IDE.

**Live transaction** refers to the testing sending a transaction to a running Fuel Client
node to exercise your Sway code. Instruction-by-instruction debugging is available in the `forc debug` CLI.

- [Debugging with CLI](./debugging_with_cli.md)
- [Debugging with IDE](./debugging_with_ide.md)
