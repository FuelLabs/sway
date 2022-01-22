# The Sway Toolchain

The Sway toolchain consists of several components.

## Forc (`forc`)

The "Fuel Orchestrator" [Forc](https://github.com/FuelLabs/sway/tree/master/forc) is our equivalent of Rust's [Cargo](https://doc.rust-lang.org/cargo/). It is the primary entry point for creating, building, testing, and deploying Sway projects. The next pages in this section will introduce how to use Forc.

### Sway Language Server (`forc lsp`)

The Sway Language Server `forc lsp` is provided to expose features to IDEs. Currently, only [Visual Studio Code is supported through a plugin](https://marketplace.visualstudio.com/items?itemName=FuelLabs.sway-vscode-plugin). Vim support is forthcoming, though [syntax highlighting is provided](https://github.com/FuelLabs/sway.vim).

Note that there is no need to manually run `forc lsp` (the plugin will automatically start it), however `forc` should be included in your `$PATH` if you are building `forc` from source.

## Fuel Core (`fuel-core`)

While not directly part of the Sway toolchain, an implementation of the Fuel protocol, [Fuel Core](https://github.com/FuelLabs/fuel-core), is provided. Note that [the SDK](https://github.com/FuelLabs/fuels-rs) will automatically start and stop an instance of the node during tests, so there is no need to manually run a node unless using Forc directly without the SDK.
