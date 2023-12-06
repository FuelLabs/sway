# From Source

The `Sway toolchain` can be built directly from the [`Sway repository`](https://github.com/FuelLabs/sway).

## Installation & Updating

<!-- markdown-link-check-disable -->
In the root of the repository `/sway/<here>` build [`forc`](https://fuellabs.github.io/sway/v0.48.0/book/forc/commands/index.html) with the following command:
<!-- markdown-link-check-enable -->

```bash
cargo build
```

The executable binary can be found in `/sway/target/debug/forc`.

## Using the Toolchain

After installing run the following command to check the version:

```bash
/sway/target/debug/forc --version
```

The output may look similar to:

```bash
forc 0.31.2
```
