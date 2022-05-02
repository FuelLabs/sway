pub const SUBHEADERS: &[&str] = &["USAGE:", "ARGS:", "OPTIONS:", "SUBCOMMANDS:"];
pub const INDEX_HEADER: &str = "Here are a list of commands available to forc:\n\n";

pub static RUN_WRITE_DOCS_MESSAGE: &str = "please run `cargo run --bin forc-documenter write-docs`. If you have made local changes to any forc native commands, please install forc from path first: `cargo install --path ./forc`, then run the command.";

pub static EXAMPLES_HEADER: &str = "\n## EXAMPLES:\n";
pub static FORC_INIT_EXAMPLE: &str = r#"
```console
$ forc init my-fuel-project
$ cd my-fuel-project
$ tree
.
├── Cargo.toml
├── Forc.toml
├── src
│   └── main.sw
└── tests
    └── harness.rs
```

`Forc.toml` is the Forc manifest file, containing information about the project and dependencies. `Cargo.toml` is the Rust project manifest file, used by the Rust-based tests package.

A `src/` directory is created, with a single `main.sw` Sway file in it.

A `tests/` directory is also created. The `Cargo.toml` in the root directory contains necessary Rust dependencies to enable you to write Rust-based tests using our Rust SDK (`fuels-rs`). More on this in the `Test` section down below.
"#;

pub static FORC_BUILD_EXAMPLE: &str = r#"
Compile the sway files of the current project.

```console
$ forc build
Compiled script "my-fuel-project".
Bytecode size is 28 bytes.
```

The output produced will depend on the project's program type. Building script, predicate and contract projects will produce their bytecode in binary format `<project-name>.bin`. Building contracts and libraries will also produce the public ABI in JSON format `<project-name>-abi.json`.

By default, these artifacts are placed in the `out/` directory.

If a `Forc.lock` file did not yet exist, it will be created in order to pin each of the dependencies listed in `Forc.toml` to a specific commit or version.
"#;

pub static FORC_TEST_EXAMPLE: &str = r#"
You can write tests in Rust using our [Rust SDK](https://github.com/FuelLabs/fuels-rs). These tests can be run using `forc test`, which will look for Rust tests under the `tests/` directory (which is created automatically with `forc init`).

You can find an example under the [Testing with Rust](../../testing/testing-with-rust.md) section.
"#;

pub static FORC_DEPLOY_EXAMPLE: &str = r#"
You can use `forc deploy`, which triggers a contract deployment transaction and sends it to a running node.

Alternatively, you can deploy your Sway contract programmatically using [fuels-rs](https://github.com/FuelLabs/fuels-rs), our Rust SDK.

You can find an example within our [fuels-rs book](https://fuellabs.github.io/fuels-rs/latest/getting-started/basics.html#deploying-a-sway-contract).
"#;

pub static FORC_PARSE_BYTECODE_EXAMPLE: &str = r#"
Example with the initial project created using `forc init`:

```console
$ forc build -o obj
Compiled script "my-fuel-project".
Bytecode size is 28 bytes.
```

```console
my-second-project$ forc parse-bytecode obj

 half-word  byte  op               raw                notes
         0  0     JI(4)            [144, 0, 0, 4]     conditionally jumps to byte 16
         1  4     NOOP             [71, 0, 0, 0]
         2  8     Undefined        [0, 0, 0, 0]       data section offset lo (0)
         3  12    Undefined        [0, 0, 0, 28]      data section offset hi (28)
         4  16    LW(46, 12, 1)    [93, 184, 192, 1]
         5  20    ADD(46, 46, 12)  [16, 186, 227, 0]
         6  24    RET(0)           [36, 0, 0, 0]
```
"#;
