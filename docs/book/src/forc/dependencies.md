# Dependencies

Forc has a dependency management system which can pull packages using `git`, `ipfs` or `registry`. This allows users to build and share Forc libraries.

## Adding a dependency

If your `Forc.toml` doesn't already have a `[dependencies]` table, add one. Below, list the package name alongside its source. Currently, `forc` supports `git`, `ipfs`, `path` and `registry` sources.

If a `git` source is specified, `forc` will fetch the git repository at the given URL and then search for a `Forc.toml` for a package with the given name anywhere inside the git repository.

The following example adds a library dependency named `custom_lib`. For git dependencies you may optionally specify a `branch`, `tag`, or `rev` (i.e. commit hash) reference.

```toml
[dependencies]
custom_lib = { git = "https://github.com/FuelLabs/custom_lib", branch = "master" }
# custom_lib = { git = "https://github.com/FuelLabs/custom_lib", tag = "v0.0.1" }
# custom_lib = { git = "https://github.com/FuelLabs/custom_lib", rev = "87f80bdf323e2d64e213895d0a639ad468f4deff" }
```

Depending on a local library using `path`:

```toml
[dependencies]
custom_lib = { path = "../custom_lib" }
```

For `ipfs` sources, `forc` will fetch the specified `cid` using either a local `ipfs` node or a public gateway. `forc` automatically tries to connect to local `ipfs` node. If it fails, it defaults to using `https://ipfs.io/` as a gateway.

The following example adds a dependency with an `ipfs` source.

```toml
[dependencies]
custom_lib = { ipfs = "QmYwAPJzv5CZsnA625s3Xf2nemtYgPpHdWEz79ojWnPbdG" }
```

For `registry` sources, `forc` will first resolve the source declared by its name and version. This is done using the forc.pub-index repo (located at `https://github.com/FuelLabs/forc.pub-index`). The package name and version is used to convert the declaration to an IPFS CID. The resolved IPFS CID is then used by either a local IPFS node or an IPFS gateway that fuel operates (`https://ipfs.forc.pub/`) to actually fetch the package.

Available packages can be found at `https://forc.pub`.

The following example adds a dependency with a `registry` source.

```toml
[dependencies]
custom_lib = "0.0.1"
```

In the example above the package `custom_lib v0.0.1` will be fetched from `forc.pub` using IPFS.

We do not currently support `offline` mode of operation for project that uses `registry` sources. Also wildcard declarations (ex: `custom_lib = *`) to get the latest version available for that package or caret declarations (ex: `custom_lib = ^0.1`) to get `SemVer` compatible latest available option for a given dependency is not supported yet.

Once the package is added, running `forc build` will automatically download added dependencies.

## Updating dependencies

To update dependencies in your Forc directory you can run `forc update`. For `path` and `ipfs` dependencies this will have no effect. For `git` dependencies with a `branch` reference, this will update the project to use the latest commit for the given branch.
