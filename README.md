# Sway

[![build](https://github.com/FuelLabs/sway/actions/workflows/ci.yml/badge.svg)](https://github.com/FuelLabs/sway/actions/workflows/ci.yml)
[![crates.io](https://img.shields.io/crates/v/forc?label=latest)](https://crates.io/crates/forc)
[![docs](https://docs.rs/forc/badge.svg)](https://docs.rs/forc/)
[![twitter](https://img.shields.io/twitter/follow/SwayLang)](https://x.com/SwayLang)
[![discord](https://img.shields.io/badge/chat%20on-discord-orange?&logo=discord&logoColor=ffffff&color=7389D8&labelColor=6A7EC2)](https://discord.gg/xfpK4Pe)

Sway is a language developed for the [Fuel](https://docs.fuel.network/docs/intro/what-is-fuel/) blockchain. It is heavily inspired by Rust and aims to bring modern language development and performance to the blockchain ecosystem.

## Documentation

For user documentation, including installing release builds, see the latest
released Sway Book: <https://fuellabs.github.io/sway/latest/>.

The documentation URLs describe different source versions:

- `latest` redirects to the most recently published Sway release.
- `vX.Y.Z` is documentation built from that exact Sway release tag.
- `master` is built from the default branch and may describe unreleased
  behavior.

These labels are Sway documentation versions. They are not Fuelup channel names
and do not identify the toolchain activated on a Fuel network. Check the
compiler you are running with `forc --version` and consult the
[Fuelup channel documentation](https://install.fuel.network/master/concepts/channels.html)
when selecting network-compatible tooling.

For Sway standard library documentation from the default branch, see
<https://fuellabs.github.io/sway/master/std/>.

Also view the default-branch technical reference for the Sway programming
language at <https://fuellabs.github.io/sway/master/reference/>.

The **Stable** Sway and Forc pages on `docs.fuel.network` are published by
[`FuelLabs/docs-hub`](https://github.com/FuelLabs/docs-hub) from an explicitly
selected Sway release. That selection can differ from both the newest upstream
release and the compiler in a named Fuelup network channel.

## Building from Source

This section is for developing the Sway compiler and toolchain. For developing contracts and using Sway, see the above documentation section.

### Dependencies

Sway is built in Rust. To begin, install the Rust toolchain following instructions at <https://www.rust-lang.org/tools/install>. Then configure your Rust toolchain to use Rust `stable`:

```sh
rustup default stable
```

If not already done, add the Cargo bin directory to your `PATH` by adding the following line to `~/.profile` and restarting the shell session.

```sh
export PATH="${HOME}/.cargo/bin:${PATH}"
```

### Building Forc

Clone the repository and build the Sway toolchain:

```sh
git clone git@github.com:FuelLabs/sway.git
cd sway
cargo build
```

Confirm the Sway toolchain built successfully:

```sh
cargo run --bin forc -- --help
```

## All other scripts/commands

For all other scripts and commands use https://github.com/casey/just:

```
> just --list
Available recipes:
    [automation]
    update-contract-ids
    update-fuel-dependencies

    [benchmark]
    benchmark
    benchmark-tests
    collect-gas-usage

    [build]
    build-highlightjs
    build-prism
    generate-sway-lib-std

    [ci]
    ci-check
    install-ci-check

    [test]
    test-forc-fmt-check-panic
```

## Contributing to Sway

We welcome contributions to Sway!

Please see the [Contributing To Sway](https://fuellabs.github.io/sway/master/book/reference/contributing_to_sway.html) section of the Sway book for guidelines and instructions to help you get started.
