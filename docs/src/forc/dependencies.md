# Dependencies

Forc has a dependency management system which can pull packages using git. This allows users to build and share Forc libraries.

## Adding a dependency

If your `Forc.toml` doesn't already have a `[dependencies]` table, add one. Below, list the package name alongside its source. Currently, `forc` supports both `git` and `path` sources.

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

Once the package is added, running `forc build` will automatically download added dependencies.

## Updating dependencies

To update dependencies in your Forc directory you can run `forc update`. For `path` dependencies this will have no effect. For `git` dependencies with a `branch` reference, this will update the project to use the latest commit for the given branch.
