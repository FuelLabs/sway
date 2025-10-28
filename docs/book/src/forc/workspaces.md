# Workspaces

A *workspace* is a collection of one or more packages, namely *workspace members*, that are managed together.

The key points for workspaces are:

* Common `forc` commands available for a single package can also be used for a workspace, like `forc build` or `forc deploy`.
* All packages share a common `Forc.lock` file which resides in the root directory of the workspace.

Workspace manifests are declared within `Forc.toml` files and support the following fields:

* [`members`](#the-members-field) - Packages to include in the workspace.
* [`[patch]`](#the-patch-section) - Defines the patches.

An empty workspace can be created with `forc new --workspace` or `forc init --workspace`.

## The `members` field

The `members` field defines which packages are members of the workspace:

```toml
[workspace]
members = ["member1", "path/to/member2"]
```

The `members` field accepts entries to be given in relative path with respect to the workspace root.
Packages that are located within a workspace directory but are *not* contained within the `members` set are ignored.

## The `[patch]` section

The `[patch]` section can be used to override any dependency in the workspace dependency graph. The usage is the same with package level `[patch]` section and details can be seen [here](./manifest_reference.md#the-patch-section).

It is not allowed to declare patch table in member of a workspace if the workspace manifest file contains a patch table.

Example with Git dependency:

```toml
[workspace]
members = ["member1", "path/to/member2"]

[patch.'https://github.com/fuellabs/sway']
std = { git = "https://github.com/fuellabs/sway", branch = "test" }
```

In the above example each occurrence of `std` as a dependency in the workspace will be changed with `std` from `test` branch of sway repo.

Example with registry dependency:

```toml
[workspace]
members = ["contract-a", "contract-b", "script"]

[patch.'forc.pub']
std = { path = "../custom-std" }
```

In this example, all workspace members will use the local custom version of `std` instead of the registry version.

## Some `forc` commands that support workspaces

* `forc build` - Builds an entire workspace.
* `forc deploy` - Builds and deploys all deployable members (i.e, contracts) of the workspace in the correct order.
* `forc run` - Builds and runs all scripts of the workspace.
* `forc check` - Checks all members of the workspace.
* `forc update` - Checks and updates workspace level `Forc.lock` file that is shared between workspace members.
* `forc clean` - Cleans all output artifacts for each member of the workspace.
* `forc fmt` - Formats all members of a workspace.
