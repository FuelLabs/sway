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
forc build -o obj
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
         ...
         ...
         ...
         60   240    Undefined            00 00 00 00
         61   244    Undefined            fa f9 0d d3
         62   248    Undefined            00 00 00 00
         63   252    Undefined            00 00 00 c8
```
"#;

pub static FORC_COMPLETIONS_EXAMPLE: &str = r#"
DISCUSSION:

Enable tab completion for Bash, Fish, Zsh, or PowerShell
The script is output on `stdout`, allowing one to re-direct the
output to the file of their choosing. Where you place the file
will depend on which shell, and which operating system you are
using. Your particular configuration may also determine where
these scripts need to be placed.

Here are some common set ups for the three supported shells under
Unix and similar operating systems (such as GNU/Linux).

**BASH:**

Completion files are commonly stored in `/etc/bash_completion.d/` for
system-wide commands, but can be stored in
`~/.local/share/bash-completion/completions` for user-specific commands.
Run the command:

```sh
mkdir -p ~/.local/share/bash-completion/completions
forc completions --shell=bash >> ~/.local/share/bash-completion/completions/forc
```

This installs the completion script. You may have to log out and
log back in to your shell session for the changes to take effect.

**BASH (macOS/Homebrew):**

Homebrew stores bash completion files within the Homebrew directory.
With the `bash-completion` brew formula installed, run the command:

```sh
mkdir -p $(brew --prefix)/etc/bash_completion.d
forc completions --shell=bash > $(brew --prefix)/etc/bash_completion.d/forc.bash-completion
```

**FISH:**

Fish completion files are commonly stored in
`$HOME/.config/fish/completions`. Run the command:

```sh
mkdir -p ~/.config/fish/completions
forc completions --shell=fish > ~/.config/fish/completions/forc.fish
```

This installs the completion script. You may have to log out and
log back in to your shell session for the changes to take effect.

**ZSH:**

ZSH completions are commonly stored in any directory listed in
your `$fpath` variable. To use these completions, you must either
add the generated script to one of those directories, or add your
own to this list.

Adding a custom directory is often the safest bet if you are
unsure of which directory to use. First create the directory; for
this example we'll create a hidden directory inside our `$HOME`
directory:

```sh
mkdir ~/.zfunc
```

Then add the following lines to your `.zshrc` just before
`compinit`:

```sh
fpath+=~/.zfunc
```

Now you can install the completions script using the following
command:

```sh
forc completions --shell=zsh > ~/.zfunc/_forc
```

You must then either log out and log back in, or simply run

```sh
exec zsh
```

for the new completions to take effect.

**CUSTOM LOCATIONS:**

Alternatively, you could save these files to the place of your
choosing, such as a custom directory inside your $HOME. Doing so
will require you to add the proper directives, such as `source`ing
inside your login script. Consult your shells documentation for
how to add such directives.

**POWERSHELL:**

The powershell completion scripts require PowerShell v5.0+ (which
comes with Windows 10, but can be downloaded separately for windows 7
or 8.1).

First, check if a profile has already been set

```sh
Test-Path $profile
```

If the above command returns `False` run the following

```sh
New-Item -path $profile -type file -force
```

Now open the file provided by `$profile` (if you used the
`New-Item` command it will be
`${env:USERPROFILE}\Documents\WindowsPowerShell\Microsoft.PowerShell_profile.ps1`

Next, we either save the completions file into our profile, or
into a separate file and source it inside our profile. To save the
completions into our profile simply use

```sh
forc completions --shell=powershell >> ${env:USERPROFILE}\Documents\WindowsPowerShell\Microsoft.PowerShell_profile.ps1
```
"#;
