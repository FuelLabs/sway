# Installation

The _Sway toolchain_ is sufficient to compile Sway smart contracts. Otherwise, note that if you want to run Sway smart contracts (e.g. for testing), a Fuel Core full node is required, which is packaged together with the _Sway toolchain_ together as the _Fuel toolchain_.

## Installing from Pre-compiled Binaries

Pre-compiled release binaries for Linux and macOS are available for the Sway toolchain. Native Windows is currently unsupported ([tracking issue for Windows support](https://github.com/FuelLabs/sway/issues/1526)). Windows Subsystem for Linux should work but is not officially supported.

[`fuelup`](https://github.com/FuelLabs/fuelup) is the equivalent of Rust's `rustup` for the Fuel toolchain. It enables easily downloading binary releases of the Fuel toolchain.

Start by installing `fuelup` with:

```sh
curl --proto '=https' --tlsv1.2 -sSf \
    https://fuellabs.github.io/fuelup/fuelup-init.sh | sh
```

`fuelup-init` will ask for permission to add `~/.fuelup/bin` to your PATH. Otherwise, you can also pass `--no-modify-path` so that `fuelup-init` does not modify your PATH:

```sh
curl --proto '=https' --tlsv1.2 -sSf \
    https://fuellabs.github.io/fuelup/fuelup-init.sh | sh -s -- --no-modify-path
```

Once `fuelup` is installed, `fuelup-init` automatically runs the command below

```sh
fuelup toolchain install latest
```

to install the latest Fuel toolchain.

You can run the same command at a later time to update the toolchain.

### Installing from Cargo

The Sway toolchain and Fuel Core full node can be installed from source with Cargo with:

```sh
cargo install forc fuel-core
```

#### Updating `forc` from Cargo

You can update the toolchain from source with Cargo with:

```sh
cargo install forc fuel-core
```

#### Installing `forc` Plugins from Cargo

The Fuel ecosystem has a few plugins which can be easily installed via Cargo.

> **Note**: `forc` detects anything in your `$PATH` prefixed with `forc-` as a plugin. Use `forc plugins` to see what you currently have installed.

```sh
# Sway Formatter
cargo install forc-fmt

# Block Explorer
cargo install forc-explore

# Sway Language Server
cargo install forc-lsp
```

## Installing from Source

### Dependencies

A prerequisite for installing and using Sway is the Rust toolchain. Platform-specific instructions for installing `rustup` can be found [here](https://www.rust-lang.org/tools/install). Then, install the Rust toolchain with:

```sh
# Install the latest stable Rust toolchain.
rustup install stable
```

Installing `fuel-core` may require installing additional system dependencies. See [here](https://github.com/FuelLabs/fuel-core#building) for instructions.

The Sway toolchain is built and tested against the `stable` Rust toolchain version (<https://github.com/rust-lang/rust/releases/latest>). There is no guarantee it will work with the `nightly` Rust toolchain, or with earlier `stable` versions, so ensure you are using `stable` with:

```sh
# Update installed Rust toolchain; can be used independently.
rustup update
# Set the stable Rust toolchain as default; can be used independently.
rustup default stable
```

### Building from Source

Rather than installing from `cargo`, the Sway toolchain can be built from a local source checkout by following instructions at <https://github.com/FuelLabs/sway>. The Fuel Core full node implementation can be built from source by following instructions at <https://github.com/FuelLabs/fuel-core>.

## Enable tab completion for Bash, Fish, Zsh, or PowerShell

`forc` supports generating completion scripts for Bash, Fish, Zsh, and PowerShell. See `forc completions --help` for full details, but the gist is as simple as using one of the following:

```sh
# Bash
forc completions --shell=bash > ~/.local/share/bash-completion/completions/forc

# Bash (macOS/Homebrew)
forc completions --shell=bash > $(brew --prefix)/etc/bash_completion.d/forc.bash-completion

# Fish
mkdir -p ~/.config/fish/completions
forc completions --shell=fish > ~/.config/fish/completions/forc.fish

# Zsh
forc completions --shell=zsh > ~/.zfunc/_forc

# PowerShell v5.0+
forc completions --shell=powershell >> $PROFILE.CurrentUserCurrentHost
# or
forc completions --shell=powershell | Out-String | Invoke-Expression
```

Once the completions have been generated and properly installed, close and reopen your terminal for the new completions to take effect.
