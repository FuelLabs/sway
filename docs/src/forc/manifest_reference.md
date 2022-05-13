# Manifest Reference

The `Forc.toml` (the _manifest_ file) is a compulsory file for each package and it is written in [TOML] format. `Forc.toml` consists of the following fields:

* [`[project]`](#the-project-section) — Defines a sway project.
  * `name` — The name of the project.
  * `authors` — The authors of the project.
  * `organization` — The organization of the project.
  * `license`— The project license.
  * `entry` — The entry point of the project.
    * For `scripts` and `predicates` point to the file with `main` function.
    * For `libraries` see [Libraries](./../sway-program-types/libraries.md).
    * For `contracts` point to the file with the `ABI` implementation.
  * `implicit_std` -  Controls whether provided `std` version (with the current `forc` version) will get added as a dependency _implicitly_. _Unless you know what you are doing, leave this as default._

* [`[dependencies]`](#the-dependencies-section) — Defines the dependencies.
* `[network]` — Defines a network for forc to interact with.
  * `url` — URL of the network.

## The `[project]` section

An example `Forc.toml` is shown below. Under `[project]` the following fields are optional:

* `authors`
* `organization`

Also for the following fields, a default value is provided so omitting them is allowed:

* `entry` - (default : _main.sw_)
* `implicit_std` - (default : _true_)

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
