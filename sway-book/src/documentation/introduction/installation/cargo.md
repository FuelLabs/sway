# Cargo

Cargo can be used to install the Sway toolchain with various [`plugins`](../../forc/plugins/index.md).

A prerequisite for installing and using Sway is the [`Rust toolchain`](https://www.rust-lang.org/tools/install).

## Dependencies

Install the Rust toolchain with:

```bash
# Install the latest stable Rust toolchain.
rustup install stable
```

## Installation & Updating

The Sway toolchain and [`Fuel Core`]((https://github.com/FuelLabs/fuel-core)) can be installed/updated with:

```bash
cargo install forc fuel-core
```

Installing [`fuel-core`](https://github.com/FuelLabs/fuel-core) may require installing additional [`system dependencies`](https://github.com/FuelLabs/fuel-core#building).
