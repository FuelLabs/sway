
# Sway Language Server

[![Visual Studio Marketplace Version](https://img.shields.io/visual-studio-marketplace/v/FuelLabs.sway-vscode-plugin)](https://marketplace.visualstudio.com/items?itemName=FuelLabs.sway-vscode-plugin)
[![discord](https://img.shields.io/badge/chat%20on-discord-orange?&logo=discord&logoColor=ffffff&color=7389D8&labelColor=6A7EC2)](https://discord.gg/xfpK4Pe)

This extension provides LSP support for the Sway smart contract programming language.

## Features

- go-to type definition 
- types and documentation on hover
- inlay hints for types and parameter names
- semantic syntax highlighting
- symbol renaming
- code actions
- imports insertion

_Coming Soon_
- code completion
- apply suggestions from errors
- find all references, workspace symbol search
- ... and many more

## Quick start

1. Install the [Fuel toolchain](https://fuellabs.github.io/fuelup/master/installation/index.html).
1. Ensure `forc-lsp` is installed correctly by entering `forc-lsp --version` into your terminal.
1. Install the [Sway VSCode plugin](https://marketplace.visualstudio.com/items?itemName=FuelLabs.sway-vscode-plugin).

## Trying out the local version

To try out the local LSP version:

1. Install the local version of the server: `cargo install --path ./forc-plugins/forc-lsp`.
1. Open VSCode settings and set the `Sway-lsp › Diagnostic: Bin Path` to the installed `forc-lsp` binary. The path to the binary will be listed at the end of the `cargo install` command and is usually: `/home/<user>/.cargo/bin/forc-lsp`.
1. Open an arbitrary Sway project. E.g., `./examples/arrays`.
1. Open the _Output_ window in VSCode and select _Sway Language Server_ from the drop down menu.
1. Start coding and observe the LSP output in the _Output_ window. This window will also show any `dbg!` or `eprintln!` lines. 