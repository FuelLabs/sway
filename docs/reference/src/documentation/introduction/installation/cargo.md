# Cargo

Cargo can be used to install the Sway toolchain with various [`plugins`](../../forc/plugins/index.md).

## Dependencies

A prerequisite for installing and using Sway is the [`Rust toolchain`](https://www.rust-lang.org/tools/install) running on the `stable` channel.

After installing the `Rust toolchain` run the following command to check default channel:

```bash
rustup toolchain list
```

The output may look similar to:

```bash
stable-x86_64-unknown-linux-gnu (default)
```

## Installation & Updating

The `Sway toolchain` can be installed/updated with:

```bash
cargo install forc
```
