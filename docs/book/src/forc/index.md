# Forc Reference

Forc stands for Fuel Orchestrator. Forc provides a variety of tools and commands for developers working with the Fuel ecosystem, such as scaffolding a new project, formatting, running scripts, deploying contracts, testing contracts, and more. If you're coming from a Rust background, forc is similar to cargo.

The core `forc` executable is released from the
[`FuelLabs/sway`](https://github.com/FuelLabs/sway) repository. Some
network-facing plugins, including `forc-client` and `forc-node`, are released
independently from [`FuelLabs/forc`](https://github.com/FuelLabs/forc).
Consequently, a Sway compiler version is not a complete plugin compatibility
matrix. Check each installed executable with `<command> --version`; when its
version differs from this book's release, prefer that executable's `--help`
output for exact flags.

If you are new to Forc, see the [Forc Project](https://docs.fuel.network/docs/sway/introduction/forc_project/) introduction section.

For a comprehensive overview of the Forc CLI commands, see the [Commands](./commands/index.md) section.
