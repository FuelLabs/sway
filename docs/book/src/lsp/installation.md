# Installation

The Sway language server is contained in the [`forc-lsp`](../forc/plugins/forc_lsp.md) binary, which is installed as part of the [Fuel toolchain](../introduction/fuel_toolchain.md). Once installed, it can be used with a variety of IDEs. It must be installed for any of the IDE plugins to work.

> **Note**: There is no need to manually run `forc-lsp` (the plugin will automatically start it), however both `forc` and `forc-lsp` must be in your `$PATH`. To check if `forc` is in your `$PATH`, type `forc --help` in your terminal.

## VSCode

This is the best supported editor at the moment.

You can install the latest release of the plugin from the [marketplace](https://marketplace.visualstudio.com/items?itemName=FuelLabs.sway-vscode-plugin).

Note that we only support the most recent version of VS Code.

## Code OSS (VSCode on Linux)

1. Install [code-marketplace](https://aur.archlinux.org/packages/code-marketplace) to get access to all of the extensions in the VSCode marketplace.
2. Install the [Sway](https://marketplace.visualstudio.com/items?itemName=FuelLabs.sway-vscode-plugin) extension.

## vim / neovim

Follow the documentation for [sway.vim](https://github.com/FuelLabs/sway.vim) to install.

## helix

[Install helix](https://docs.helix-editor.com/install.html) and Sway LSP will work out of the box.

Sway support is built into helix using [tree-sitter-sway](https://github.com/FuelLabs/tree-sitter-sway).

## Emacs

Coming soon! Feel free to [contribute](https://github.com/FuelLabs/sway/issues/3527).
