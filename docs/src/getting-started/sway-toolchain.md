# The Sway Toolchain

The Sway toolchain consists of several components.

## `forc`

The "Fuel Orchestrator" `forc` is our equivalent of Rust's [Cargo](https://doc.rust-lang.org/cargo/). It is the primary entry point for creating, building, testing, and deploying Sway projects. The next pages in this section will introduce how to use `forc`.

## `sway-server`

The Sway Language Server `sway-server` is provided to expose features to IDEs. Currently, only Visual Studio Code is supported. Vim support is forthcoming. Note that there is no need to manually run `sway-server`, however it should be included in your `$PATH`.

## `fuel-core`

While not directly part of the Sway toolchain, an implementation of the Fuel full node, `fuel-core`, is provided. Note that for now, users must manually run `fuel-core` to deploy contracts or run scripts. In the future, an instance of `fuel-core` will be initialized through `forc`.
