# Plugins

Plugins can be used to extend `forc` with new commands that go beyond the native commands mentioned in the previous chapter. While the Fuel ecosystem provides a few commonly useful plugins (`forc-fmt`, `forc-client`, `forc-lsp`, `forc-migrate`), anyone can write their own!

Let's install a plugin, `forc-install`, and see what's underneath the plugin:

```sh
git clone https://github.com/darthbenro008/forc-install
cd forc-install
cargo install --path .
```

Check that we have installed `forc-install`:

```console
$ forc plugins
Installed Plugins:
forc-install
```

`forc-install` is a tool to manage GitHub dependencies in your Forc.toml file: For example, to install a sway library hosted on github:

```console
forc install https://github.com/user/sway-library
```

Note that some plugin crates can also provide more than one command. For example, installing the `forc-client` plugin provides the `forc deploy` and `forc run` commands. This is achieved by specifying multiple `[[bin]]` targets within the `forc-client` manifest.

## Writing your own plugin

We encourage anyone to write and publish their own `forc` plugin to enhance their development experience.

Your plugin must be named in the format `forc-<MY_PLUGIN>` and you may use the above template as a starting point. You can use [clap](https://docs.rs/clap/latest/clap/) and add more subcommands, options and configurations to suit your plugin's needs.
