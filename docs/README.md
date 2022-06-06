# Documentation

## Building From Source

Install `mdbook` and then open a new terminal session in order to run the subsequent commands

```sh
cargo install mdbook
```

To set up and build the book locally, you must also have `mdbook-forc-documenter` preprocessor installed, the pre-existing forc plugins in the book as well as any new plugins you want to add to the book. If changes have to be made to the Forc commands or plugins chapters, please read the next section first.

From the project root, run:

```sh
cargo install --path ./scripts/mdbook-forc-documenter
cargo install --path ./forc-plugins/forc-fmt
cargo install --path ./forc-plugins/forc-lsp
cargo install --path ./forc-plugins/forc-explore
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

The `mdbook-forc-documenter` [preprocessor](https://rust-lang.github.io/mdBook/for_developers/preprocessors.html) now automatically handles documenting forc commands and plugins, but some actions have to be taken for the preprocessor to work. Please read the [mdbook-forc-documenter README](../scripts/mdbook-forc-documenter/README.md) before making changes to Forc commands or plugins.

**It is important to note that changing the chapter names `Commands` and `Plugins` will affect the behavior of the preprocessor**. When renaming the chapters, please make the same change [here](https://github.com/FuelLabs/sway/blob/a19681c2165402d289bc6bae7a46a580ef3be5b5/scripts/mdbook-forc-documenter/src/lib.rs#L45,L56).
