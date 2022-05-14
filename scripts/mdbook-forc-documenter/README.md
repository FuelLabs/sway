# mdbook-forc-preprocessor

A preprocessor for [mdBook](https://github.com/rust-lang/mdBook) to update the Forc commands section of the Sway book based on the output we get when running `forc --help`.

This preprocessor is automatically run on every build, as long as the `book.toml` file contains the preprocessor:

```toml
[preprocessor.forc-documenter]
```

## Usage

### Adding a new forc command

Enter a new entry under the `Commands` section within `SUMMARY.md`, in this format:

```md
- [forc gm](./forc_gm.md)\n
```

### Removing a forc command

Delete the forc command entry from `SUMMARY.md`.

### Adding an example

Create a new Markdown file within `scripts/mdbook-forc-documenter/examples`, named after the desired forc command in snake case. The preprocessor automatically detects the example if there is a matching forc command with the same name as the file name, and includes it in the build.

### Removing an example

Delete the Markdown file from within the above examples directory.


