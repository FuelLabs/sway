# Plugins

Plugins can be used to extend `forc` with new commands that go beyond the native commands mentioned in the previous chapter. While the Fuel ecosystem provides a few commonly useful plugins (`forc-fmt`, `forc-lsp`, `forc-explore`), anyone can write their own!

## Writing your own plugin

We encourage anyone to write and publish their own `forc` plugin to enhance their development experience.

Your plugin must be named in the format `forc-<MY_PLUGIN>` and you may use the above template as a starting point. You can use [clap](https://docs.rs/clap/latest/clap/) and add more subcommands, options and configurations to suit your plugin's needs.
