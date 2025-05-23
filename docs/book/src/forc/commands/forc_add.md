# forc add

Adds one or more dependencies to a `Forc.toml` manifest.

## **Usage**

```bash
forc add [OPTIONS] <DEP_SPEC>...
```

## **Arguments**

* `<DEP_SPEC>`: List of dependencies in the format `name[@version]` (e.g., `custom_lib@0.1.0`, `custom_contract`)

## **Options**

* `--path <PATH>`: Add a local path dependency.
* `--git <URI>`: Add a Git-based dependency.

  * Can be combined with one of:

    * `--branch <branch>`
    * `--tag <tag>`
    * `--rev <rev>`
* `--ipfs <CID>`: Add a dependency sourced from IPFS.
* `--contract-dep`: Add to `[contract-dependencies]` instead of `[dependencies]`.
* `--salt <SALT>`: Salt to use for contract deployment (only applies to contract dependencies).
* `--package <SPEC>`: Apply change to a specific package in a workspace.
* `--manifest-path <PATH>`: Path to the `Forc.toml`.
* `--dry-run`: Show what would be changed without writing to the file.
* `--offline`: Do not fetch any remote dependencies.
* `--ipfs-node <FUEL|PUBLIC|LOCAL|URL>`: IPFS node to use for IPFS-sourced dependencies.
