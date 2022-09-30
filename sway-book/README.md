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
cargo install --path ./forc-plugins/forc-fmt
cargo install --path ./forc-plugins/forc-lsp
cargo install --path ./forc-plugins/forc-explore
```

To build book:

```sh
mdbook build
```

To build the book on strict mode to check if pages should be removed or added within the Forc Reference:

```sh
MDBOOK_preprocessor__FORC_documenter__STRICT="true" mdbook build docs
```

To serve locally:

```sh
mdbook serve
```
