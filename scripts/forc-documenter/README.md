# Forc Documenter

Forc documenter is a script to update the Forc commands section of the Sway book based on the output we get when running `forc --help`. It does the following 3 things:

1. Create a new page in the Sway book for a forc command, showing descriptions, usage, options and examples (if any)
2. Updates the index.md for the commands section to list all the available commands
3. Updates SUMMARY.md to reflect the updates in the commands

## Prerequisites

Since the script uses `forc`, make sure you have the latest Forc version installed:

```rust
// install the latest version of forc
cargo install forc
```

## Usage

You can run the script in `--dry-run` mode for it to tell you if there were changes within commands that were not updated, without writing any files:

```rust
cargo run --bin forc-documenter write-docs --dry-run
```

In the case of an inconsistency, you will receive a prompt to rebuild docs:

```console
cargo run --bin forc-documenter write-docs --dry-run
    Finished dev [unoptimized + debuginfo] target(s) in 0.24s
    Running `target/debug/forc-documenter write-docs --dry-run`
forc addr2line: documentation ok.
Error: Documentation inconsistent for forc build - please run `cargo run --bin forc-documenter write-docs`
```

The above is the same command that runs within CI to ensure docs are updated.

You can use the script without options to update the docs and write the affected files:

```rust
cargo run --bin forc-documenter write-docs
```

This is the output you will see, should the script run successfully:

```console
cargo run --bin forc-documenter write-docs
    Finished dev [unoptimized + debuginfo] target(s) in 0.25s
    Running `target/debug/forc-documenter write-docs`
Generating docs for command: forc addr2line...
Generating docs for command: forc build...
Generating docs for command: forc clean...
Generating docs for command: forc completions...
Generating docs for command: forc deploy...
Generating docs for command: forc explorer...
Generating docs for command: forc fmt...
Generating docs for command: forc init...
Generating docs for command: forc json-abi...
Generating docs for command: forc lsp...
Generating docs for command: forc parse-bytecode...
Generating docs for command: forc run...
Generating docs for command: forc test...
Generating docs for command: forc update...
Updating forc commands in forc/commands/index.md...
Updating forc commands in SUMMARY.md...
Done.
```
