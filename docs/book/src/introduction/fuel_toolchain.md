# The Fuel Toolchain

The Fuel toolchain consists of several components.

## Forc (`forc`)

The "Fuel Orchestrator" [Forc](https://github.com/FuelLabs/sway/tree/master/forc) is our equivalent of Rust's [Cargo](https://doc.rust-lang.org/cargo/). It is the primary entry point for creating, building, testing, and deploying Sway projects.

## Sway Language Server (`forc-lsp`)

The Sway Language Server `forc-lsp` is provided to expose features to IDEs. [Installation instructions](./installation.md).

Currently, only [Visual Studio Code is supported through a plugin](https://marketplace.visualstudio.com/items?itemName=FuelLabs.sway-vscode-plugin). Vim support is forthcoming, though [syntax highlighting is provided](https://github.com/FuelLabs/sway.vim).

> **Note**: There is no need to manually run `forc-lsp` (the plugin will automatically start it), however both `forc` and `forc-lsp` must be in your `$PATH`. To check if `forc` is in your `$PATH`, type `forc --help` in your terminal.

## Sway Formatter (`forc-fmt`)

A canonical formatter is provided with `forc-fmt`. [Installation instructions](./installation.md). It can be run manually with

```sh
forc fmt
```

The [Visual Studio Code plugin](https://marketplace.visualstudio.com/items?itemName=FuelLabs.sway-vscode-plugin) will
automatically format Sway files with `forc-fmt` on save, though you might have to explicitly set the Sway plugin as the
default formatter, like this:

```json
"[sway]": {
  "editor.defaultFormatter": "FuelLabs.sway-vscode-plugin"
}
```

## Fuel Core (`fuel-core`)

An implementation of the Fuel protocol, [Fuel Core](https://github.com/FuelLabs/fuel-core), is provided together with the _Sway toolchain_ to form the _Fuel toolchain_. [The Rust SDK](https://github.com/FuelLabs/fuels-rs) will automatically start and stop an instance of the node during tests, so there is no need to manually run a node unless using Forc directly without the SDK.
