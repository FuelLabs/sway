# forc remove

Removes one or more dependencies from a `Forc.toml` manifest.

## **Usage**

```bash
forc remove [OPTIONS] <DEP_SPEC>...
```

## **Arguments**

* `<DEP_SPEC>`: List of dependencies to remove by name (e.g., `custom_lib`, `custom_contract`)

## **Options**

* `--contract-dep`: Remove from `[contract-dependencies]` instead of `[dependencies]`.
* `--package <SPEC>`: Target a specific package in a workspace.
* `--manifest-path <PATH>`: Path to the `Forc.toml`.
* `--dry-run`: Preview what would be removed without making any changes.
* `--offline`: Prevent forc from fetching metadata or resolving versions remotely.
* `--ipfs-node <FUEL|PUBLIC|LOCAL|URL>`: IPFS node to use for reference.
