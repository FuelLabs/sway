# Documentation

## Building From Source

To build book:

```sh
mdbook build
```

To serve locally:

```sh
mdbook serve
```

## Regenerate Forc SubCommand Docs

With forc installed running the command

```sh
cargo run --bin forc-documenter write-docs
```

will generate the proper docs for `forc` and its commands based on `forc --help`. This behavior is further documented in [the Forc documenter README](../scripts/forc-documenter/README.md).
