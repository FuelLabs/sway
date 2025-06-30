# Documentation

## Building From Source

Install `mdbook` and then open a new terminal session in order to run the subsequent commands

```sh
cargo install mdbook
```

To set up and build the book locally, you must also have `mdbook-forc-documenter` preprocessor and relevant forc plugins installed.

If you wish to make changes to the `Commands` or `Plugins` chapters, please read the [next section](#generating-documentation-for-forc-commandsplugins) first.

From the project root, install `mdbook-forc-documenter`:

```sh
cargo install --path ./scripts/mdbook-forc-documenter
```

You must also install forc plugins that are already documented within the book. You can skip plugins that are going to be removed and install plugins that are going to be added to the book:

```sh
cargo install --path ./forc-plugins/forc-client
cargo install --path ./forc-plugins/forc-doc
cargo install --path ./forc-plugins/forc-fmt
cargo install --path ./forc-plugins/forc-lsp
```

To build book:

```sh
mdbook build docs/book
```

To build the book on strict mode to check if pages should be removed or added within the Forc Reference:

```sh
MDBOOK_preprocessor__FORC_documenter__STRICT="true" mdbook build docs/book
```

To serve locally:

```sh
mdbook serve docs/book
```

## Generating documentation for Forc commands/plugins

The `mdbook-forc-documenter` [preprocessor](https://rust-lang.github.io/mdBook/for_developers/preprocessors.html) now automatically handles documenting forc commands and plugins, but some actions have to be taken for the preprocessor to work. Please read the [mdbook-forc-documenter README](../../scripts/mdbook-forc-documenter/README.md) before making changes to Forc commands or plugins.

**It is important to note that changing the chapter names `Commands` and `Plugins` will affect the behavior of the preprocessor**. When renaming the chapters, please make the same change [here](https://github.com/FuelLabs/sway/blob/a19681c2165402d289bc6bae7a46a580ef3be5b5/scripts/mdbook-forc-documenter/src/lib.rs#L45,L56).
