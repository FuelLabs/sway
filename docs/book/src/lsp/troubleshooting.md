# Troubleshooting

First, confirm you are running the most recent version:

```sh
fuelup toolchain install latest
fuelup update
forc-lsp --version
```

Second, confirm that your `$PATH` resolves to the `forc-lsp` binary in `$HOME/.fuelup/bin`.

```sh
which forc-lsp
```

## Slow Performance

If you are experiencing slow performance, you can try the following:

Follow [the steps above](#troubleshooting) to ensure you are running the most recent version.

Then, make sure you only have the most recent version of the LSP server running.

```sh
pkill forc-lsp
```

### Large projects

Sway projects with ten or more Sway files are likely to have slower LSP performance. We are working on better support for large projects.

In the meantime, if it's too slow, you can disable the LSP server entirely with the `sway-lsp.diagnostic.disableLsp` setting. The extension will still provide basic syntax highlighting, command palettes, as well as the Sway debugger, but all other language features will be disabled.

## Server Logs

You can enable verbose logging of the LSP server.

In VSCode, this is under the setting:

```json
"sway-lsp.trace.server": "verbose"
```

Once enabled, you can find this in the output window under Sway Language Server.

For other editors, see [Installation](./installation.md) for links to documentation.
