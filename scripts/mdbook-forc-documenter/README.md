# mdbook-forc-preprocessor

A preprocessor for [mdBook](https://github.com/rust-lang/mdBook) to update the Forc commands and plugins section of the Sway book based on the output we get when running `forc --help`.

This preprocessor is automatically run on every build, as long as the `book.toml` file contains the preprocessor:

```toml
[preprocessor.forc-documenter]
```

The preprocessor runs with strict mode **off** by default to enable building the book regardless of errors in the Forc Reference pages. To check if pages should be added or removed, run with the `strict` [environment variable](https://rust-lang.github.io/mdBook/format/configuration/environment-variables.html):

```sh
MDBOOK_preprocessor__FORC_documenter__STRICT="true" mdbook build docs/book
```

## Usage

### Adding a new forc command

Enter a new entry under the `Commands` section within `SUMMARY.md`, in this format:

```md
- [forc fmt](./forc_fmt.md)
```

### Adding a new forc plugin

Do the same as the above, with an extra step of adding an installation step within the CI. The preprocessor needs to be aware of the plugin when building the book, since it is calling `forc <plugin> --help` to generate the documentation. You can add this installation step within [`ci.yml`](https://github.com/FuelLabs/sway/blob/a19681c2165402d289bc6bae7a46a580ef3be5b5/.github/workflows/ci.yml#L126) and [`gh-pages.yml`](https://github.com/FuelLabs/sway/blob/a19681c2165402d289bc6bae7a46a580ef3be5b5/.github/workflows/gh-pages.yml#L26).

### Removing a forc command

Delete the entry from `SUMMARY.md`.

### Removing a forc plugin

Do the same as the above, with an extra step of removing the installation command within `ci.yml` and `gh-pages.yml`.

### Adding an example

Create a new Markdown file within `scripts/mdbook-forc-documenter/examples`, named after the desired forc command or plugin in snake case. The preprocessor automatically detects the example if there is a matching forc command or plugin with the same name as the file name, and includes it in the build.

### Removing an example

Delete the Markdown file from within the above examples directory.
