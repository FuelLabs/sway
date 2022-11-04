## Workspaces

A *workspace* is a collection of one or more packages, called *workspace
members*, that are managed together.

The key points of workspaces are:

* Common commands can run across all workspace members, like `forc build` or `forc deploy`.
* All packages share a common [`Forc.lock`] file which resides in the
  *workspace root*.

For workspaces, `Forc.toml` accepts following fields:

* `members`(#the-members-field) - Packages to include in the workspace. 

### The `members` field

The `members` field define which packages are members of
the workspace:

```toml
members = ["member1", "path/to/member2"]
```

The `members` field accepts entries to be given in relative path with respect to the workspace root. 


### Some forc commands that supports workspaces

* `forc build` - Builds an entire workspace.
* `forc deploy` - Builds and deploys all deployable members (i.e, contracts) of the workspace in the correct order.
* `forc check` - Checks all members of the workspace.
* `forc update` - Checks and updates workspace level `Forc.lock` file that is shared between workspace members.
