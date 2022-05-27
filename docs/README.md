# Documentation

## Building From Source

Install `mdbook` and then open a new terminal session in order to run the subsequent commands

```sh
cargo install mdbook
```

To build book:

```sh
mdbook build
```

To serve locally:

```sh
mdbook serve
```

## Generating documentation for Forc commands/plugins

The `mdbook-forc-documenter` now automatically handles documenting forc commands and plugins. This behavior is further documented in [the mdbook-forc-documenter README](../scripts/mdbook-forc-documenter/README.md).

**It is important to note that changing the chapter names `Commands` and `Plugins` will affect the behaviour of the preprocessor**. When renaming the chapters, please make the same change [here](https://github.com/FuelLabs/sway/blob/master/scripts/mdbook-forc-documenter/src/lib.rs#L45,L56).
