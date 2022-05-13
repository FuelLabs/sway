# forc-init
Create a new Forc project


## USAGE:
forc init [OPTIONS] <PROJECT_NAME>


## ARGS:

<_PROJECT_NAME_>

   The name of your project


## OPTIONS:

`--contract` 

The default program type, excluding all flags or adding this flag
creates a basic contract program

`-h`, `--help` 

Print help information

`--library` 

Adding this flag creates an empty library program

`--predicate` 

Adding this flag creates an empty predicate program

`--script` 

Adding this flag creates an empty script program

`-t`, `--template` <_TEMPLATE_>

Initialize a new project from a template.

Example Templates:
- counter

## EXAMPLES:

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