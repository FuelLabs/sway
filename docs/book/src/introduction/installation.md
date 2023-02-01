# Installation

The _Sway toolchain_ is sufficient to compile Sway smart contracts. Otherwise, note that if you want to run Sway smart contracts (e.g. for testing), a Fuel Core full node is required, which is packaged together with the _Sway toolchain_ together as the _Fuel toolchain_.

## Install via Pre-compiled Binaries (Recommended)

Installing via pre-compiled release binaries is the recommended way to get up and running with the Sway toolchain. Pre-compiled binaries for Linux and macOS are available. Native Windows is currently unsupported ([tracking issue for Windows support](https://github.com/FuelLabs/sway/issues/1526)). Windows Subsystem for Linux should work but is not officially supported.

[`fuelup`](https://github.com/FuelLabs/fuelup) is the equivalent of Rust's `rustup` for the Fuel toolchain. It enables easily downloading binary releases of the Fuel toolchain.

1. Start by installing `fuelup` with the following command:

   ```sh
   curl --proto '=https' --tlsv1.2 -sSf \
   https://fuellabs.github.io/fuelup/fuelup-init.sh | sh
   ```

   This downloads the `fuelup-init` script to a temp directory on your machine, which installs `fuelup`. `fuelup-init` will ask for permission to add `~/.fuelup/bin` to your PATH. Otherwise, you can also pass `--no-modify-path` so that `fuelup-init` does not modify your PATH:

   ```sh
   curl --proto '=https' --tlsv1.2 -sSf \
   https://fuellabs.github.io/fuelup/fuelup-init.sh | sh -s -- --no-modify-path
   ```

2. Once fuelup is installed, fuelup-init automatically runs `fuelup toolchain install latest` to install the latest toolchain

   You can run `fuelup update` at anytime to get the most up-to-date toolchain.

3. (Optional) You can optionally install distributed toolchains optimized for different networks.

   To configure the optimal toolchain for beta-2, run the following commands:

   ```sh
   $ fuelup self update
   Fetching binary from https://github.com/FuelLabs/fuelup/releases/download/v0.16.1/fuelup-0.16.1-aarch64-apple-darwin.tar.gz
    Downloading component fuelup without verifying checksum
    Unpacking and moving fuelup to /var/folders/tp/0l8zdx9j4s9_n609ykwxl0qw0000gn/T/.tmpP3HfvR
    Moving /var/folders/tp/0l8zdx9j4s9_n609ykwxl0qw0000gn/T/.tmpP3HfvR/fuelup to /Users/user/.fuelup/bin/fuelup


   $ fuelup toolchain install beta-2
   Downloading: forc forc-explore forc-wallet fuel-core fuel-indexer

   Adding component forc v0.31.1 to 'beta-2-aarch64-apple-darwin'
   Fetching binary from https://github.com/FuelLabs/sway/releases/download/v0.31.1/forc-binaries-darwin_arm64.tar.gz
   npacking and moving forc-doc to /Users/user/.fuelup/toolchains/beta-2-aarch64-apple-darwin/bin
   Unpacking and moving forc to /Users/user/.fuelup/toolchains/beta-2-aarch64-apple-darwin/bin
   Unpacking and moving forc-deploy to /Users/user/.fuelup/toolchains/beta-2-aarch64-apple-darwin/bin
   Unpacking and moving forc-run to /Users/user/.fuelup/toolchains/beta-2-aarch64-apple-darwin/bin
   Unpacking and moving forc-lsp to /Users/user/.fuelup/toolchains/beta-2-aarch64-apple-darwin/bin
   Unpacking and moving forc-fmt to /Users/user/.fuelup/toolchains/beta-2-aarch64-apple-darwin/bin
   Fetching core forc dependencies
   Installed forc v0.31.1 for toolchain 'beta-2-aarch64-apple-darwin'

   Adding component forc-explore v0.28.1 to 'beta-2-aarch64-apple-darwin'
   Fetching binary from https://github.com/FuelLabs/forc-explorer/releases/download/v0.28.1/forc-explore-0.28.1-aarch64-apple-darwin.tar.gz
   Unpacking and moving forc-explore to /Users/user/.fuelup/toolchains/beta-2-aarch64-apple-darwin/bin
   Installed forc-explore v0.28.1 for toolchain 'beta-2-aarch64-apple-darwin'

   Adding component forc-wallet v0.1.2 to 'beta-2-aarch64-apple-darwin'
   Fetching binary from https://github.com/FuelLabs/forc-wallet/releases/download/v0.1.2/forc-wallet-0.1.2-aarch64-apple-darwin.tar.gz
   Unpacking and moving forc-wallet to /Users/user/.fuelup/toolchains/beta-2-aarch64-apple-darwin/bin
   Installed forc-wallet v0.1.2 for toolchain 'beta-2-aarch64-apple-darwin'

   Adding component fuel-core v0.15.3 to 'beta-2-aarch64-apple-darwin'
   Fetching binary from https://github.com/FuelLabs/fuel-core/releases/download/v0.15.3/fuel-core-0.15.3-aarch64-apple-darwin.tar.gz
   Unpacking and moving fuel-core to /Users/user/.fuelup/toolchains/    beta-2-aarch64-apple-darwin/bin
   Installed fuel-core v0.15.3 for toolchain 'beta-2-aarch64-apple-darwin'

   Adding component fuel-indexer v0.1.13 to 'beta-2-aarch64-apple-darwin'
   Fetching binary from https://github.com/FuelLabs/fuel-indexer/releases/download/v0.1.13/fuel-indexer-0.1.13-aarch64-apple-darwin.tar.gz
   Unpacking and moving fuel-indexer to /Users/user/.fuelup/toolchains/beta-2-aarch64-apple-darwin/bin
   Installed fuel-indexer v0.1.13 for toolchain 'beta-2-aarch64-apple-darwin'

   Installed:
   - forc 0.31.1
   - forc-explore 0.28.1
   - forc-wallet 0.1.2
   - fuel-core 0.15.3
   - fuel-indexer 0.1.13

   The Fuel toolchain is installed and up to date
   ```

You're all set to start building!

### Need Help?

You may refer to [The Fuelup Book](https://fuellabs.github.io/fuelup/latest/) for an in-depth look into fuelup, or check out the tooling section in the [Fuel forum](https://forum.fuel.network/) if you're running into problems through the installation process. If you don't see your question, post the issue you're running into with as many details as possible and the team will get back to you asap!

### Installing from Cargo

The Sway toolchain and Fuel Core full node can be installed from source with Cargo with:

```sh
cargo install forc fuel-core
```

#### Updating `forc` from Cargo

You can update the toolchain from source with Cargo with:

```sh
cargo update forc fuel-core
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
