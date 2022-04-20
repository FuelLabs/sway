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

`forc` and `fuel-core` are built and tested against the `stable` Rust toolchain version 1.58 or later. If your install fails the first time, use `rustup update` and try again.

There is no guarantee that either package will work with the `nightly` Rust toolchain, so ensure you are using `stable` with:

```sh
rustup default stable
```

### Updating `forc`

You can update `forc` and `fuel-core` with:

```sh
cargo install forc fuel-core
```

### Installing `forc` Plugins

The Fuel ecosystem has a few plugins which can be easily installed via cargo.

Note, `forc` detects anything in your path prefixed with `forc-` as a plugin. Use `forc plugins` to see what you currently have installed.

```sh
# Sway Formatter
cargo install forc-fmt

# Block Explorer
cargo install forc-explore

# Forc Language Server
cargo install forc-lsp
```

## Building from Source

The Sway toolchain can be built from source by following instructions at <https://github.com/FuelLabs/sway>.

The Fuel Core full node implementation can be built from source by following instructions at <https://github.com/FuelLabs/fuel-core>.

## Enable tab completion for Bash, Fish, Zsh, or PowerShell

`forc` now supports generating completion scripts for Bash, Fish, Zsh, and PowerShell. See `forc completions --help` for full details, but the gist is as simple as using one of the following:

```
# Bash
$ forc completions --shell=bash > ~/.local/share/bash-completion/completions/forc

# Bash (macOS/Homebrew)
$ forc completions --shell=bash > $(brew --prefix)/etc/bash_completion.d/forc.bash-completion

# Fish
$ mkdir -p ~/.config/fish/completions
$ forc completions --shell=fish > ~/.config/fish/completions/forc.fish

# Zsh
$ forc completions --shell=zsh > ~/.zfunc/_forc

# PowerShell v5.0+
$ forc completions --shell=powershell >> $PROFILE.CurrentUserCurrentHost
# or
$ forc completions --shell=powershell | Out-String | Invoke-Expression
```

Once the completions have been generated and properly installed, close and reopen your terminal for the new completions to take effect.

