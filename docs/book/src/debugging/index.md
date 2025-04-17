# Debugging

Forc provides tools for debugging both live transactions as well as Sway unit tests.
Debugging can be done via CLI or using the VSCode IDE.

**Unit testing** refers to "in-language" test functions annotated with `#[test]`. Line-by-line
debugging is available within the VSCode IDE.

**Live transaction** refers to the testing sending a transaction to a running Fuel Client
node to exercise your Sway code. Instruction-by-instruction debugging is available in the `forc debug` CLI.

- [Debugging with CLI](./debugging_with_cli.md)
- [Debugging with IDE](./debugging_with_ide.md)

## `__dbg` intrinsic function

Sway also offers the `__dbg` intrinsic function to help debug all applications types: scripts, contracts and predicates.
When called, this intrinsic function will print the current file, line and column, together with a customizable print of the specified value.

```sway
script;
fn main() -> u64 {
    __dbg(1u64)
}
```

The application above will print:

```terminal
[src/main.sw:3:5] = 1
```

Structs can be customized by implementing the `Debug` trait.

```sway
script;
struct S { }
impl Debug for S {
    fn fmt(self, ref mut f: Formatter) {
        f.debug_struct("S2")
            .field("field1", 1)
            .field("field2", "Hello")
            .finish();
    }
}
fn main() -> u64 {
    let _ = __dbg(S {});
    __dbg(1u64)
}
```

This code is very similar to what the Sway compiler generates by default for all declared types.
And this is what is printed:

```terminal
[src/main.sw:12:13] = S2 { field1: 1, field2: "Hello" }
[src/main.sw:13:5] = 1
```
