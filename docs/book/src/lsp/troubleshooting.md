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

## Logs

You can you enable verbose logging of the LSP server. 

In VSCode, this is under the setting:

```json
"sway-lsp.trace.server": "verbose"
```

Once enabled, you can find this in the output window under Sway Language Server.

For other editors, see [Installation](./installation.md) for links to documentation.
