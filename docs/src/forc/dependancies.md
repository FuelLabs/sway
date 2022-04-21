# Dependencies

Forc has a dependency management system which can pull packages using git. This allows users to build and share Forc libraries.

## Adding a dependency

If your `Forc.toml` doesn't already have a `[dependencies]` table, add one. Below, list the package name alongside its source. Currently, `forc` supports both `git` and `path` sources.

When installing from `git` Forc will look for the nearest `Forc.toml` with a matching project `name` field to the dependancy name specified.

This example adds a dependency of the custom crate, you can specify a specific git `branch`, `tag` or `rev` field:

```toml
[dependencies]
custom_lib = { git = "https://github.com/FuelLabs/custom_lib", branch = "master" }
# custom_lib = { git = "https://github.com/FuelLabs/custom_lib", tag = "v0.0.1" }
# custom_lib = { git = "https://github.com/FuelLabs/custom_lib", rev = "87f80bdf323e2d64e213895d0a639ad468f4deff" }
```

Installing a local library using `path`:

```toml
[dependencies]
custom_lib = { path = "../custom_lib" }
```

Once the package is added, you can re-run `forc build` to include and build the project.

## Updating dependencies

To update dependancies in your Forc directory you can run `forc update`.
