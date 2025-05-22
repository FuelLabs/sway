# Dependencies

Forc has a dependency management system which can pull packages using `git`, `ipfs`, `path`, or the community `registry`. This allows users to build and share Forc libraries.

## Adding Dependencies

You can add dependencies manually in your `Forc.toml`, or by using the `forc add` command.

### Using `forc add`

The `forc add` CLI supports various sources and optional flags:

```bash
forc add <dep> [--path <PATH>] [--git <URL> --tag <TAG>] [--ipfs <CID>] [--contract-dep]
```

#### Add Examples

* From a Git branch:

  ```bash
  forc add custom_lib --git https://github.com/FuelLabs/custom_lib --branch master
  ```

* From a local path:

  ```bash
  forc add custom_lib --path ../custom_lib
  ```

* From IPFS:

  ```bash
  forc add custom_lib --ipfs QmYwAPJzv5CZsnA...
  ```

* From registry (forc.pub):

  ```bash
  forc add custom_lib@0.0.1
  ```

* Add as a contract dependency:

  ```bash
  forc add my_contract --git https://github.com/example/contract --contract-dep
  ```

Optional:

* `--salt <HEX>` for custom contract salt.
* `--package <NAME>` to target a specific package in a workspace.
* `--manifest-path <PATH>` to specify a manifest file.

> ⚠️ **Note:**
> We do not currently support offline mode for projects that use **registry** sources.
> Also wildcard declarations `(ex: custom_lib = *)` to get the latest version available for that package or caret declarations `(ex: custom_lib = ^0.1)` to get `SemVer` compatible latest available option for a given dependency is not supported yet.

Once the package is added, running `forc build` will automatically fetch and resolve the dependencies.

### Manually Editing `Forc.toml`

If your `Forc.toml` doesn't already have a `[dependencies]` or `[contract-dependencies]` table, add one. Below, list the package name and its source.

#### Local Path

```toml
[dependencies]
custom_lib = { path = "../custom_lib" }
```

#### IPFS Source

```toml
[dependencies]
custom_lib = { ipfs = "QmYwAPJzv5CZsnA..." }
```

#### Registry Source (forc.pub)

```toml
[dependencies]
custom_lib = "0.0.1"
```

## Removing Dependencies

You can remove one or more dependencies using the `forc remove` command:

```bash
forc remove <dep> [--contract-dep] [--package <NAME>] [--manifest-path <PATH>]
```

### Remove Examples

* Remove from `[dependencies]`:

  ```bash
  forc remove custom_lib
  ```

* Remove from `[contract-dependencies]`:

  ```bash
  forc remove my_contract --contract-dep
  ```

* Target a specific package in a workspace:

  ```bash
  forc remove custom_lib --package my_project
  ```

## Updating Dependencies

To update dependencies in your Forc directory you can run:

```bash
forc update
```

For path and ipfs dependencies this will have no effect. For git dependencies with a branch reference, this will update the project to use the latest commit for the given branch.
