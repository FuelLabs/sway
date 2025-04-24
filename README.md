# Sway

[![build](https://github.com/FuelLabs/sway/actions/workflows/ci.yml/badge.svg)](https://github.com/FuelLabs/sway/actions/workflows/ci.yml)
[![crates.io](https://img.shields.io/crates/v/forc?label=latest)](https://crates.io/crates/forc)
[![docs](https://docs.rs/forc/badge.svg)](https://docs.rs/forc/)
[![license](https://img.shields.io/github/license/FuelLabs/sway)](https://github.com/FuelLabs/sway/blob/master/LICENSE)
[![twitter](https://img.shields.io/twitter/follow/SwayLang)](https://x.com/SwayLang)
[![discord](https://img.shields.io/badge/chat%20on-discord-orange?&logo=discord&logoColor=ffffff&color=7389D8&labelColor=6A7EC2)](https://discord.gg/xfpK4Pe)

Sway is a language developed for the [Fuel](https://docs.fuel.network/docs/intro/what-is-fuel/) blockchain. It is heavily inspired by Rust and aims to bring modern language development and performance to the blockchain ecosystem.

## Documentation

For user documentation, including installing release builds, see the Sway Book: <https://fuellabs.github.io/sway/latest/>.

For Sway Standard library documentation, see: <https://fuellabs.github.io/sway/master/std/>

Also view the technical reference for the Sway programming language: <https://fuellabs.github.io/sway/master/reference/>

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

## Contributing to Sway

We welcome contributions to Sway!

Please see the [Contributing To Sway](https://fuellabs.github.io/sway/master/book/reference/contributing_to_sway.html) section of the Sway book for guidelines and instructions to help you get started.
