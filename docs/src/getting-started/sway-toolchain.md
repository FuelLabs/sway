# The Sway Toolchain

The Sway toolchain consists of several components.

## Forc (`forc`)

The "Fuel Orchestrator" Forc is our equivalent of Rust's [Cargo](https://doc.rust-lang.org/cargo/). It is the primary entry point for creating, building, testing, and deploying Sway projects. The next pages in this section will introduce how to use Forc.

## Sway Language Server `sway-server`

The Sway Language Server `sway-server` is provided to expose features to IDEs. Currently, only [Visual Studio Code is supported through a plugin](https://github.com/FuelLabs/sway-vscode-plugin). Vim support is forthcoming, though [syntax highlighting is provided](https://github.com/FuelLabs/sway.vim). Note that there is no need to manually run `sway-server`, however it should be included in your `$PATH`.

## Fuel Core (`fuel-core`)

While not directly part of the Sway toolchain, an implementation of the Fuel protocol, `fuel-core`, is provided. Note that for now, users must manually run `fuel-core` to deploy contracts or run scripts. In the future, an instance of `fuel-core` will be initialized through `forc`.
