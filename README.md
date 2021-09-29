# Sway

[![Compile and Test](https://github.com/FuelLabs/sway/actions/workflows/cargo_test.yml/badge.svg)](https://github.com/FuelLabs/sway/actions/workflows/cargo_test.yml)
[![Community](https://img.shields.io/badge/chat%20on-discord-orange?&logo=discord&logoColor=ffffff&color=7389D8&labelColor=6A7EC2)](https://discord.gg/xfpK4Pe)

Sway is a language developed for the Fuel blockchain. It is heavily inspired by Rust and aims to bring modern language development and performance to the blockchain ecosystem.

## Documentation

For user documentation, see the Sway Book: <https://fuellabs.github.io/sway/latest/>.

## Building from Source

### Dependencies

Sway is built in Rust. To begin, install the Rust toolchain following instructions at <https://www.rust-lang.org/tools/install>. Then configure your Rust toolchain to use Rust `stable`:

```console
rustup default stable
```

To ensure access to all dependent repositories, [create](https://docs.github.com/en/authentication/connecting-to-github-with-ssh/generating-a-new-ssh-key-and-adding-it-to-the-ssh-agent) and [add](https://docs.github.com/en/authentication/connecting-to-github-with-ssh/adding-a-new-ssh-key-to-your-github-account) SSH keys to your GitHub account.

## Building Sway

Clone the repository and build the Sway toolchain:

```console
git clone git@github.com:FuelLabs/sway.git
cd sway
cargo build
```

Confirm the Sway toolchain build successfully:

```console
cargo run --bin forc -- --help
# or
./target/debug/forc --help
```

To run `forc` from any directory, add `<SWAY_REPO_PATH>/target/debug/` to your `$PATH`.
