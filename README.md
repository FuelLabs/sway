# Sway

[![Compile and Test](https://github.com/FuelLabs/sway/actions/workflows/cargo_test.yml/badge.svg)](https://github.com/FuelLabs/sway/actions/workflows/cargo_test.yml)
[![Community](https://img.shields.io/badge/chat%20on-discord-orange?&logo=discord&logoColor=ffffff&color=7389D8&labelColor=6A7EC2)](https://discord.gg/xfpK4Pe)

Sway is a language developed for the Fuel blockchain. It is heavily inspired by Rust and aims to bring modern language development and performance to the blockchain ecosystem.

## Documentation

For user documentation, see the Sway Book: <https://fuellabs.github.io/sway/latest/>.

## Building from Source

### Dependencies

Sway is built in Rust. To begin, install the Rust toolchain following instructions at <https://www.rust-lang.org/tools/install>. Then configure your Rust toolchain to use Rust `stable`:

```sh
rustup default stable
```

If not already done, add the Cargo bin directory to your `PATH` by adding the following line to `~/.profile` and restarting the shell session.

```sh
export PATH="${HOME}/.cargo/bin:${PATH}"
```

To ensure access to all dependent repositories, [create](https://docs.github.com/en/authentication/connecting-to-github-with-ssh/generating-a-new-ssh-key-and-adding-it-to-the-ssh-agent) and [add](https://docs.github.com/en/authentication/connecting-to-github-with-ssh/adding-a-new-ssh-key-to-your-github-account) SSH keys to your GitHub account.

### Building Sway

Clone the repository and build the Sway toolchain:

```sh
git clone git@github.com:FuelLabs/sway.git
cd sway
cargo build
```

Confirm the Sway toolchain built successfully:

```sh
cargo run --bin forc -- --help
# or
./target/debug/forc --help
```

To run `forc` from any directory, install `forc` to your local Cargo bin directory:

```sh
cargo install --locked --path forc
# Also install sway-server if using the IDE plugin
cargo install --locked --path sway-server
```
