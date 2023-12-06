# Installation

The `Sway toolchain` is required to compile `Sway` programs.

There are three ways to install the `Sway toolchain`:

- [`Fuelup`](fuelup.md)
- [`Cargo`](cargo.md)
- [`From Source`](source.md)

The supported operating systems include Linux and macOS; however, Windows is [`unsupported`](https://github.com/FuelLabs/sway/issues/1526).

## Fuelup

[`Fuelup`](fuelup.md) is the recommended tool for installation and management of the toolchain.

## Cargo

`Cargo` may be used instead of [`Fuelup`](fuelup.md); however, the user needs to manage the toolchain themselves.

<!-- markdown-link-check-disable -->
The advantage of using `Cargo` is the installation of [`plugins`](https://fuellabs.github.io/sway/v0.48.0/book/forc/plugins/index.html) that have not been added into [`Fuelup`](fuelup.md).

The disadvantage occurs when [`Fuelup`](fuelup.md) and `Cargo` are used in tandem because the latest [`plugins`](https://fuellabs.github.io/sway/v0.48.0/book/forc/plugins/index.html) may not be recognized.
<!-- markdown-link-check-enable -->

## Source

The latest features may be accessed when installing from [`source`](source.md); however, the features may not be ready for release and lead to unstable behavior.
