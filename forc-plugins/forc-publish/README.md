# forc-publish

Forc subcommand for uploading a package to the registry.

## Authentication

Requires either the `--token` argument to be passed, the environment variable `FORC_PUB_TOKEN`, or a `~/.forc/credentials.toml` file like this:

```toml
[registry]
token = "YOUR_TOKEN"
```

This credential file can be created automatically by running the CLI without the `--token` argument.

## Local development

For local development, run [forc.pub](https://github.com/FuelLabs/forc.pub), create an API token and then `forc publish --registry-url http://localhost:8080`
