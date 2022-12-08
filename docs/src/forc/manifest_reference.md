# Manifest Reference

The `Forc.toml` (the _manifest_ file) is a compulsory file for each package and it is written in [TOML] format. `Forc.toml` consists of the following fields:

* [`[project]`](#the-project-section) — Defines a sway project.
  * `name` — The name of the project.
  * `authors` — The authors of the project.
  * `organization` — The organization of the project.
  * `license`— The project license.
  * `entry` — The entry point for the compiler to start parsing from.
    * For the recomended way of selecting an entry point of large libraries please take a look at: [Libraries](./../sway-program-types/libraries.md)
  * `implicit-std` -  Controls whether provided `std` version (with the current `forc` version) will get added as a dependency _implicitly_. _Unless you know what you are doing, leave this as default._
  * `forc-version` - The minimum forc version required for this project to work properly.

* [`[dependencies]`](#the-dependencies-section) — Defines the dependencies.
* `[network]` — Defines a network for forc to interact with.
  * `url` — URL of the network.

* [`[build-profiles]`](#the-build-profiles--section) - Defines the build profiles.

* [`[patch]`](#the-patch-section) - Defines the patches.

* [`[contract-dependencies]`](#the-contract-dependencies-section) - Defines the contract dependencies.

## The `[project]` section

An example `Forc.toml` is shown below. Under `[project]` the following fields are optional:

* `authors`
* `organization`

Also for the following fields, a default value is provided so omitting them is allowed:

* `entry` - (default : _main.sw_)
* `implicit-std` - (default : _true_)

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

## The `[build-profiles-*]` section

The `[build-profiles]` tables provide a way to customize compiler settings such as debug options.

The following fields needs to be provided for a build-profile:

* `print-ast` - Whether to print out the generated AST (true) or not (false).
* `print-dca-graph` - Whether to print out the computed DCA graph (in GraphViz DOT format).
* `print-finalized-asm` - Whether to compile to bytecode (false) or to print out the generated ASM (true).
* `print-intermediate-asm` - Whether to compile to bytecode (false) or to print out the generated ASM (true).
* `print-ir` - Whether to compile to bytecode (false) or to print out the generated IR (true).
* `terse-mode` - Terse mode. Limited warning and error output.

There are two default `[build-profile]` available with every manifest file. These are `debug` and `release` profiles. If you want to override these profiles, you can provide them explicitly in the manifest file like the following example:

```toml
[project]
authors = ["user"]
entry = "main.sw"
organization = "Fuel_Labs"
license = "Apache-2.0"
name = "wallet_contract"

[build-profiles.debug]
print-finalized-asm = false
print-intermediate-asm = false
print-ir = false
terse = false

[build-profiles.release]
print-finalized-asm = false 
print-intermediate-asm = false
print-ir = false
terse = true
```

Since `release` and `debug` implicitly included in every manifest file, you can use them by just passing `--release` or by not passing anything (debug is default). For using a user defined build profile there is `--build-profile <profile name>` option available to the relevant commands. (For an example see [forc-build](../forc/commands/forc_build.md))

Note that providing the corresponding cli options (like `--print-finalized-asm`) will override the selected build profile. For example if you pass both `--release` and `--print-finalized-asm`, release build profile is omitted and resulting build profile would have a structure like the following:

* print-finalized-asm - true
* print-intermediate-asm - false
* print-ir - false
* terse - false

## The `[patch]` section

The [patch] section of `Forc.toml` can be used to override dependencies with other copies. The example provided below patches <https://github.com/fuellabs/sway> source with master branch of the same repo.

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

The `[contract-dependenices]` table can be used to declare contract dependencies for a Sway contract or script. Contract dependencies are the set of contracts that our contract or script may interact with. Declaring `[contract-dependencies]` makes it easier to refer to contracts in your Sway source code without having to manually update IDs each time a new version is deployed. Instead, we can use forc to pin and update contract dependencies just like we do for regular library dependencies.

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

Because the ID of a contract is computed deterministically, rebuilding the same contract would always result in the same contract ID. Since two contracts with same contract ID cannot be deployed on the blockchain, a "salt" factor is needed to modify the contract ID. For each contract dependency declared under `[contract-dependencies]`, `salt` can be specified. An example is shown below:

```toml
[contract-dependencies]
foo = { path = "../foo", salt = "0x1000000000000000000000000000000000000000000000000000000000000000" }
```

For contract dependencies that do not specify any value for `salt`, a default of all zeros for `salt` is implicitly applied.
