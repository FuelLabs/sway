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
We can try this command with the initial project created using `forc init`, with the counter template:

```sh
forc init --template counter counter
cd counter
```

```
$ forc build -o obj
  Creating a new `Forc.lock` file. (Cause: missing path info for dependency: std)
    Adding core
    Adding std git+https://github.com/fuellabs/sway?tag=v0.11.0#95816e4e41aae1d3425ba6ff5e7266076d8400fa
   Created new lock file at /Users/user/Projects/fuel/counter/Forc.lock
  Compiled library "core".
  Compiled library "std".
  Compiled contract "counter".
  Bytecode size is 256 bytes.
```

```console
counter$ forc parse-bytecode obj

  half-word   byte   op                   raw           notes
          0   0      JI(4)                90 00 00 04   conditionally jumps to byte 16
          1   4      NOOP                 47 00 00 00
          2   8      Undefined            00 00 00 00   data section offset lo (0)
          3   12     Undefined            00 00 00 c8   data section offset hi (200)
          4   16     LW(63, 12, 1)        5d fc c0 01
          5   20     ADD(63, 63, 12)      10 ff f3 00
          6   24     LW(17, 6, 73)        5d 44 60 49
          7   28     LW(16, 63, 4)        5d 43 f0 04
          8   32     EQ(16, 17, 16)       13 41 14 00
          9   36     Undefined            73 40 00 0e
         10   40     LW(16, 63, 5)        5d 43 f0 05
         11   44     EQ(16, 17, 16)       13 41 14 00
         12   48     Undefined            73 40 00 19
         13   52     RVRT(0)              36 00 00 00
         14   56     MOVE(19, 5)          1a 4c 50 00
         15   60     CFEI(32)             91 00 00 20
         16   64     LW(18, 6, 74)        5d 48 60 4a
         17   68     ADDI(16, 19, 0)      50 41 30 00
         18   72     LW(17, 63, 6)        5d 47 f0 06
         19   76     ADD(17, 17, 12)      10 45 13 00
         20   80     ADDI(16, 19, 0)      50 41 30 00
         21   84     MCPI(16, 17, 32)     60 41 10 20
         22   88     ADDI(16, 19, 0)      50 41 30 00
         23   92     SWW(16, 18)          3a 41 20 00
         24   96     RET(18)              24 48 00 00
         25   100    MOVE(18, 5)          1a 48 50 00
         26   104    CFEI(72)             91 00 00 48
         27   108    LW(17, 6, 74)        5d 44 60 4a
         28   112    ADDI(16, 18, 8)      50 41 20 08
         29   116    LW(16, 63, 6)        5d 43 f0 06
         30   120    ADD(16, 16, 12)      10 41 03 00
         31   124    ADDI(19, 18, 8)      50 4d 20 08
         32   128    MCPI(19, 16, 32)     60 4d 00 20
         33   132    ADDI(16, 18, 8)      50 41 20 08
         34   136    SRW(16, 16)          38 41 00 00
         35   140    ADD(17, 17, 16)      10 45 14 00
         36   144    ADDI(16, 18, 0)      50 41 20 00
         37   148    SW(18, 17, 0)        5f 49 10 00
         38   152    ADDI(16, 18, 0)      50 41 20 00
         39   156    LW(19, 18, 0)        5d 4d 20 00
         40   160    ADDI(16, 18, 40)     50 41 20 28
         41   164    LW(17, 63, 6)        5d 47 f0 06
         42   168    ADD(17, 17, 12)      10 45 13 00
         43   172    ADDI(16, 18, 40)     50 41 20 28
         44   176    MCPI(16, 17, 32)     60 41 10 20
         45   180    ADDI(16, 18, 40)     50 41 20 28
         46   184    SWW(16, 19)          3a 41 30 00
         47   188    ADDI(16, 18, 0)      50 41 20 00
         48   192    LW(16, 18, 0)        5d 41 20 00
         49   196    RET(16)              24 40 00 00
         50   200    Undefined            f3 83 b0 ce
         51   204    ANDI(13, 24, 3045)   51 35 8b e5
         52   208    Undefined            7d aa 3b 72
         53   212    SW(57, 4, 2765)      5f e4 4a cd
         54   216    Undefined            b2 d8 80 60
         55   220    Undefined            4e 36 71 99
         56   224    Undefined            08 0b 43 79
         57   228    Undefined            c4 1b b6 ed
         58   232    Undefined            00 00 00 00
         59   236    Undefined            ab 64 e5 f2
         60   240    Undefined            00 00 00 00
         61   244    Undefined            fa f9 0d d3
         62   248    Undefined            00 00 00 00
         63   252    Undefined            00 00 00 c8
```
"#;
