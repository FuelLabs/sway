# Workspaces

A *workspace* is a collection of one or more packages, namely *workspace members*, that are managed together.

The key points for workspaces are:

* Common `forc` commands available for a single package can also be used for a workspace, like `forc build` or `forc deploy`.
* All packages share a common `Forc.lock` file which resides in the root directory of the workspace.

Workspace manifests are declared within `Forc.toml` files and support the following fields:

* [`members`](#the-members-field) - Packages to include in the workspace.

An empty workspace can be created with `forc new --workspace` or `forc init --workspace`.

## The `members` field

The `members` field defines which packages are members of the workspace:

```toml
[workspace]
members = ["member1", "path/to/member2"]
```

The `members` field accepts entries to be given in relative path with respect to the workspace root.
Packages that are located within a workspace directory but are *not* contained within the `members` set are ignored.

## Some `forc` commands that support workspaces

* `forc build` - Builds an entire workspace.
* `forc deploy` - Builds and deploys all deployable members (i.e, contracts) of the workspace in the correct order.
* `forc check` - Checks all members of the workspace.
* `forc update` - Checks and updates workspace level `Forc.lock` file that is shared between workspace members.
