# Plugins

Plugins can be used to extend `forc` with new commands that go beyond the native commands mentioned in the previous chapter. while the fuel ecosystem provides a few commonly useful plugins (`forc-fmt`, `forc-client`, `forc-lsp`, `forc-explore`), anyone can write their own!

let's install a plugin, `forc-explore`, and see what's underneath the plugin:

```sh
cargo install forc-explore
```

check that we have installed `forc-explore`:

```console
$ forc plugins
installed plugins:
forc-explore
```

`forc-explore` runs the fuel network explorer, which you can run and check out for yourself:

```console
$ forc explore
fuel network explorer 0.1.1
running server on http://127.0.0.1:3030
server::run{addr=127.0.0.1:3030}: listening on http://127.0.0.1:3030
```

you can visit <http://127.0.0.1:3030> to check out the network explorer!

note that some plugin crates can also provide more than one command. for example, installing the `forc-client` plugin provides the `forc deploy` and `forc run` commands. this is achieved by specifying multiple `[[bin]]` targets within the `forc-client` manifest.

## writing your own plugin

we encourage anyone to write and publish their own `forc` plugin to enhance their development experience.
