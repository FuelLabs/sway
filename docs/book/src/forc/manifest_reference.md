# Manifest Reference

The `Forc.toml` (the _manifest_ file) is a compulsory file for each package and it is written in [TOML] format. `Forc.toml` consists of the following fields:

* [`[project]`](#the-project-section) — Defines a sway project.
  * `name` — The name of the project.
  * `authors` — The authors of the project.
  * `organization` — The organization of the project.
  * `license`— The project license.
  * `entry` — The entry point for the compiler to start parsing from.
    * For the recommended way of selecting an entry point of large libraries please take a look at: [Libraries](./../sway-program-types/libraries.md)
  * `implicit-std` -  Controls whether provided `std` version (with the current `forc` version) will get added as a dependency _implicitly_. _Unless you know what you are doing, leave this as default._
  * `forc-version` - The minimum forc version required for this project to work properly.

* [`[dependencies]`](#the-dependencies-section) — Defines the dependencies.
* `[network]` — Defines a network for forc to interact with.
  * `url` — URL of the network.

* [`[build-profile]`](#the-build-profile-section) - Defines the build profiles.

* [`[patch]`](#the-patch-section) - Defines the patches.

* [`[contract-dependencies]`](#the-contract-dependencies-section) - Defines the contract dependencies.

## The `[project]` section

An example `Forc.toml` is shown below. Under `[project]` the following fields are optional:

* `authors`
* `organization`

Also for the following fields, a default value is provided so omitting them is allowed:

* `entry` - (default : `main.sw` )
* `implicit-std` - (default : `true` )

```toml
[project]
authors = ["user"]
entry = "main.sw"
organization = "Fuel_Labs"
license = "Apache-2.0"
name = "wallet_contract"
```

## The `[dependencies]` section

The following fields can be provided with a dependency:

* `version` - Desired version of the dependency
* `path` - The path of the dependency (if it is local)
* `git` - The URL of the git repo hosting the dependency
* `branch` - The desired branch to fetch from the git repo
* `tag` - The desired tag to fetch from the git repo
* `rev` - The desired rev (i.e. commit hash) reference

Please see [dependencies](./dependencies.md) for details

## The `[network]` section

For the following fields, a default value is provided so omitting them is allowed:

* `URL` - (default: _<http://127.0.0.1:4000>_)

## The `[build-profile.*]` section

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
* `json_abi_with_callpaths` - Whether to generate a JSON ABI with `callpaths` instead of names for structs and enums, defaults to false. This option can help prevent conflicting struct or enum definitions by using the full path instead of the name.
* `error_on_warnings` - Whether to treat errors as warnings, defaults to false.

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
json-abi-with-callpaths = false
error-on-warnings = false
experimental-private-modules = false
```

## The `[patch]` section

The [patch] section of `Forc.toml` can be used to override dependencies with other copies. The example provided below patches `https://github.com/fuellabs/sway` with the `test` branch of the same repo.

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

Just like `std` or `core` you can also patch dependencies you declared with a git repo.

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

Note that each key after the `[patch]` is a URL of the source that is being patched.

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
