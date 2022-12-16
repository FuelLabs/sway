# From Source

The `Sway toolchain` can be built directly from the [`Sway repository`](https://github.com/FuelLabs/sway).

## Installation & Updating

In the root of the repository `/sway/<here>` build [`forc`](../../forc/commands/index.md) with the following command:

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
