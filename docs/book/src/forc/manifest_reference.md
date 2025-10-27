# Manifest Reference

The `Forc.toml` (the _manifest_ file) is a compulsory file for each package and it is written in [TOML] format. `Forc.toml` consists of the following fields:

* [`[project]`](#the-project-section) — Defines a sway project.
  * `name` — The name of the project.
  * `version` — The version of the project.
  * `description` — A description of the project.
  * `authors` — The authors of the project.
  * `organization` — The organization of the project.
  * `license` — The project license.
  * `homepage` — URL of the project homepage.
  * `repository` — URL of the project source repository.
  * `documentation` — URL of the project documentation.
  * `categories` —  Categories of the project.
  * `keywords` —  Keywords the project.
  * `entry` — The entry point for the compiler to start parsing from.
    * For the recommended way of selecting an entry point of large libraries please take a look at: [Libraries](./../sway-program-types/libraries.md)
  * `implicit-std` -  Controls whether provided `std` version (with the current `forc` version) will get added as a dependency _implicitly_. _Unless you know what you are doing, leave this as default._
  * `forc-version` - The minimum forc version required for this project to work properly.
  * `metadata` - Metadata for the project; can be used by tools which would like to store package configuration in `Forc.toml`.
  * `experimental` - [Experimental features](../reference/experimental_features.md) enabled or disabled during the compilation.

* [`[dependencies]`](#the-dependencies-section) — Defines the dependencies.
* `[network]` — Defines a network for forc to interact with.
  * `url` — URL of the network.

* [`[build-profile]`](#the-build-profile-section) - Defines the build profiles.

* [`[patch]`](#the-patch-section) - Defines the patches.

* [`[contract-dependencies]`](#the-contract-dependencies-section) - Defines the contract dependencies.

## The `[project]` Section

An example `Forc.toml` is shown below. Under `[project]` the following fields are optional:

* `authors`
* `organization`
* `version`
* `description`
* `homepage`
* `repository`
* `documentation`
* `categories`
* `keywords`
* `experimental`

Also for the following fields, a default value is provided so omitting them is allowed:

* `entry` - (default : `main.sw` )
* `implicit-std` - (default : `true` )

```toml
[project]
authors = ["user"]
entry = "main.sw"
description = "Wallet contract"
version = "1.0.0"
homepage = "https://example.com/"
repository = "https://example.com/"
documentation = "https://example.com/"
organization = "Fuel_Labs"
license = "Apache-2.0"
name = "wallet_contract"
categories = ["example"]
keywords = ["example"]
experimental = { some_feature = true, some_other_feature = false }

[project.metadata]
indexing = { namespace = "counter-contract", schema_path = "out/release/counter-contract-abi.json" }
```

### Metadata Section in `Forc.toml`

The `[project.metadata]` section provides a dedicated space for external tools and plugins to store their configuration in `Forc.toml`. The metadata key names are arbitrary and do not need to match the tool's name.

#### Workspace vs Project Metadata

Metadata can be defined at two levels:

Workspace level - defined in the workspace\'s root `Forc.toml`:

```toml
[workspace.metadata]
my_tool = { shared_setting = "value" }
```

Project level - defined in individual project\'s `Forc.toml`:

```toml
[project.metadata.any_name_here]
option1 = "value"
option2 = "value"

[project.metadata.my_custom_config]
setting1 = "value"
setting2 = "value"
```

Example for an indexing tool:

```toml
[project.metadata.indexing]
namespace = "counter-contract"
schema_path = "out/release/counter-contract-abi.json"
```

When both workspace and project metadata exist:

* Project-level metadata should take precedence over workspace metadata
* Tools can choose to merge workspace and project settings
* Consider documenting your tool's metadata inheritance behavior

#### Guidelines for Plugin Developers

Best Practices

* Choose clear, descriptive metadata key names
* Document the exact metadata key name your tool expects
* Don't require `Forc.toml` if tool can function without it
* Consider using TOML format for dedicated config files
* Specify how your tool handles workspace vs project metadata

Implementation Notes

* The metadata section is optional
* Forc does not parse metadata contents
* Plugin developers handle their own configuration parsing
* Choose unique metadata keys to avoid conflicts with other tools

#### Example Use Cases

* Documentation generation settings
* Formatter configurations
* Debugger options
* Wallet integration
* Contract indexing
* Testing frameworks

This allows for a streamlined developer experience while maintaining clear separation between core Forc functionality and third-party tools.

#### External Tooling Examples

* [forc-index-ts](https://github.com/FuelLabs/example-forc-plugins/tree/master/forc-index-ts): A TypeScript CLI tool for parsing `Forc.toml` metadata to read contract ABI JSON file.
* [forc-index-rs](https://github.com/FuelLabs/example-forc-plugins/tree/master/forc-index-rs): A Rust CLI tool for parsing `Forc.toml` metadata to read contract ABI JSON file.

## The `[dependencies]` Section

The following fields can be provided with a dependency:

* `version` - Desired version of the dependency
* `namespace` - If the specified registry source (by its version) also has a namespace associated with it (optional)
* `path` - The path of the dependency (if it is local)
* `git` - The URL of the git repo hosting the dependency
* `branch` - The desired branch to fetch from the git repo
* `tag` - The desired tag to fetch from the git repo
* `rev` - The desired rev (i.e. commit hash) reference

Please see [dependencies](./dependencies.md) for details

## The `[network]` Section

For the following fields, a default value is provided so omitting them is allowed:

* `URL` - (default: _<http://127.0.0.1:4000>_)

## The `[build-profile.*]` Section

The `[build-profile]` tables provide a way to customize compiler settings such as debug options.

The following fields can be provided for a build-profile:

* `print-ast` - Whether to print out the generated AST or not, defaults to false.
* `print-dca-graph` - Whether to print out the computed Dead Code Analysis (DCA) graph (in GraphViz DOT format), defaults to false.
* `print-dca-graph-url-format` - The URL format to be used in the generated DOT file, an example for VS Code would be: `vscode://file/{path}:{line}:{col}`.
* `print-ir` - Whether to print out the generated Sway IR (Intermediate Representation) or not, defaults to false.
* `print-asm` - Whether to print out the generated ASM (assembler), defaults to false.
* `terse` - Terse mode. Limited warning and error output, defaults to false.
* `time_phases` - Whether to output the time elapsed over each part of the compilation process, defaults to false.
* `include_tests` -  Whether or not to include test functions in parsing, type-checking, and code generation. This is set to true by invocations like `forc test`, but defaults to false.
* `error_on_warnings` - Whether to treat errors as warnings, defaults to false.
* `backtrace` - Defines which panicking functions to include in a `panic` backtrace. Possible values are `all`, `all_except_never`, `only_always`, and `none`. Defaults to `all_except_never` and `only_always` in the default `debug` and `release` profiles, respectively. For more information on backtracing see the chapter [Irrecoverable Errors](../basics/error_handling.md#irrecoverable-errors).

There are two default `[build-profile]` available with every manifest file. These are `debug` and `release` profiles. If you want to override these profiles, you can provide them explicitly in the manifest file like the following example:

```toml
[project]
authors = ["user"]
entry = "main.sw"
organization = "Fuel_Labs"
license = "Apache-2.0"
name = "wallet_contract"

[build-profile.debug]
print-asm = { virtual = false, allocated = false, final = true }
print-ir = { initial = false, final = true, modified = false, passes = []}
terse = false

[build-profile.release]
print-asm = { virtual = true, allocated = false, final = true }
print-ir = { initial = true, final = false, modified = true, passes = ["dce", "sroa"]}
terse = true
```

Since `release` and `debug` are implicitly included in every manifest file, you can use them by just passing `--release` or by not passing anything (`debug` is default). For using a user defined build profile there is `--build-profile <profile name>` option available to the relevant commands. (For an example see [forc-build](../forc/commands/forc_build.md))

Note that providing the corresponding CLI options (like `--asm`) will override the selected build profile. For example if you pass both `--release` and `--asm all`, `release` build profile is overridden and resulting build profile would have a structure like the following:

```toml
print-ast = false
print-ir = { initial = false, final = false, modified = false, passes = []}
print-asm = { virtual = true, allocated = true, final = true }
terse = false
time-phases = false
include-tests = false
error-on-warnings = false
experimental-private-modules = false
```

## The `[patch]` Section

The `[patch]` section of `Forc.toml` can be used to override dependencies with other copies. This is useful for testing local changes, using unreleased features, or debugging dependencies.

### Patching Git Dependencies

The example provided below patches `https://github.com/fuellabs/sway` with the `test` branch of the same repo.

```toml
[project]
authors = ["user"]
entry = "main.sw"
organization = "Fuel_Labs"
license = "Apache-2.0"
name = "wallet_contract"

[dependencies]

[patch.'https://github.com/fuellabs/sway']
std = { git = "https://github.com/fuellabs/sway", branch = "test" }
```

In the example above, `std` is patched with the `test` branch from `std` repo. You can also patch git dependencies with dependencies defined with a path.

```toml
[patch.'https://github.com/fuellabs/sway']
std = { path = "/path/to/local_std_version" }
```

Just like `std` you can also patch dependencies you declared with a git repo.

```toml
[project]
authors = ["user"]
entry = "main.sw"
organization = "Fuel_Labs"
license = "Apache-2.0"
name = "wallet_contract"

[dependencies]
foo = { git = "https://github.com/foo/foo", branch = "master" }

[patch.'https://github.com/foo']
foo = { git = "https://github.com/foo/foo", branch = "test" }
```

### Patching Registry Dependencies

You can also patch dependencies from the registry (forc.pub) in the same way. This is particularly useful for testing local changes to `std` or other registry packages.

```toml
[project]
authors = ["user"]
entry = "main.sw"
license = "Apache-2.0"
name = "my_contract"

[dependencies]
std = "0.70.1"

[patch.'forc.pub']
std = { path = "../sway/sway-lib-std" }
```

In the example above, even though `std` version `0.70.1` would normally be fetched from the registry, the local path version is used instead.

You can also patch registry dependencies with a git repository:

```toml
[dependencies]
std = "0.70.1"

[patch.'forc.pub']
std = { git = "https://github.com/fuellabs/sway", branch = "my-feature" }
```

### Important Notes

- **Quotes are required**: Each key after `[patch]` must be in quotes (e.g., `[patch.'forc.pub']`) because they contain special characters. Without quotes, TOML will interpret dots as nested tables.
- **Source matching**: Git patches match Git dependencies, and registry patches match registry dependencies.

## The `[contract-dependencies]` section

The `[contract-dependencies]` table can be used to declare contract dependencies for a Sway contract or script. Contract dependencies are the set of contracts that our contract or script may interact with. Declaring `[contract-dependencies]` makes it easier to refer to contracts in your Sway source code without having to manually update IDs each time a new version is deployed. Instead, we can use forc to pin and update contract dependencies just like we do for regular library dependencies.

Contracts declared under `[contract-dependencies]` are built and pinned just like regular `[dependencies]` however rather than importing each contract dependency's entire public namespace we instead import their respective contract IDs as `CONTRACT_ID` constants available via each contract dependency's namespace root. This means you can use a contract dependency's ID as if it were declared as a `pub const` in the root of the contract dependency package as demonstrated in the example below.

Entries under `[contract-dependencies]` can be declared in the same way that `[dependencies]` can be declared. That is, they can refer to the `path` or `git` source of another contract. Note that entries under `[contract-dependencies]` must refer to contracts and will otherwise produce an error.

Example `Forc.toml`:

```toml
[project]
authors = ["user"]
entry = "main.sw"
organization = "Fuel_Labs"
license = "Apache-2.0"
name = "wallet_contract"

[contract-dependencies]
foo = { path = "../foo" }
```

Example usage:

```sway
script;

fn main() {
  let foo_id = foo::CONTRACT_ID;
}
```

Because the ID of a contract is computed deterministically, rebuilding the same contract would always result in the same contract ID. Since two contracts with the same contract ID cannot be deployed on the blockchain, a "salt" factor is needed to modify the contract ID. For each contract dependency declared under `[contract-dependencies]`, `salt` can be specified. An example is shown below:

```toml
[contract-dependencies]
foo = { path = "../foo", salt = "0x1000000000000000000000000000000000000000000000000000000000000000" }
```

For contract dependencies that do not specify any value for `salt`, a default of all zeros for `salt` is implicitly applied.
