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

## Generating Forc commands documentation

The `mdbook-forc-documenter` now automatically handles documenting forc commands. This behavior is further documented in [the mdbook-forc-documenter README](../scripts/mdbook-forc-documenter/README.md).
