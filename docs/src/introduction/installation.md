# Installation

Note that if you want to run (e.g. for testing) Sway smart contracts, a Fuel Core full node is required. Otherwise, the Sway toolchain is sufficient to compile Sway smart contracts.

## Dependencies

A prerequisite for installing and using Sway is the Rust toolchain. Platform-specific instructions can be found [here](https://www.rust-lang.org/tools/install).

Installing `fuel-core` may require installing additional system dependencies. See [here](https://github.com/FuelLabs/fuel-core#building) for instructions.

## Installing from Cargo

The Sway toolchain and Fuel Core full node can be installed with:

```sh
cargo install forc fuel-core
```

### Updating `forc`

You can update `forc` with:

```sh
cargo install forc
```

## Building from Source

The Sway toolchain can be built from source by following instructions at <https://github.com/FuelLabs/sway>.

The Fuel Core full node implementation can be built from source by following instructions at <https://github.com/FuelLabs/fuel-core>.
