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

## Server Logs

You can you enable verbose logging of the LSP server.

In VSCode, this is under the setting:

```json
"sway-lsp.trace.server": "verbose"
```

Once enabled, you can find this in the output window under Sway Language Server.

For other editors, see [Installation](./installation.md) for links to documentation.
