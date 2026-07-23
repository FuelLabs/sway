# Documentation update summary

## Why this update was needed

The public documentation used `latest`, release tags, `master`, Fuelup network
channels, and docs-hub's Stable label without clearly distinguishing them.
Release templates also selected different Rust SDK generations depending on
whether the generated Sway program was a contract, script, or predicate. In
addition, docs-hub cannot move to network Forc `0.70.2` until Sway publishes
the corresponding versioned generated book.

## What changed

- Documented the difference between Sway `latest`, exact `vX.Y.Z` books,
  default-branch documentation, and Fuelup network channels.
- Added warnings where the Sway Book links to legacy Sway Applications.
- Explained the split between core Forc and independently released
  network-facing plugins.
- Aligned all contract, script, and predicate Rust test templates with the
  `fuels` version exercised by the repository's SDK harness.
- Added CI checks that fail when a template is missing, unlisted, unparsable,
  or uses a different SDK generation.
- Added an exact-tag manual documentation backfill workflow.
- Prevented an empty manual dispatch from publishing a feature branch as
  `master` documentation.
- Used in-tree plugins for pre-split releases and exact independent plugin tags
  for Sway `v0.71.2`; unknown post-split release combinations fail closed.
- Ensured a historical backfill does not move the public `latest` redirect.

## Publication step

After this workflow change is merged, manually run the `github pages` workflow
with `version=v0.70.2`. Docs-hub can then update its stable Sway source and
generated-book pointers without publishing `v0.69.0` command output under a
newer label.

## Validation

- Sway Book and language-reference builds passed.
- Sway Book tests passed.
- Workflow YAML parsing and shell syntax checks passed.
- Template versions match the root SDK harness dependency.
- All referenced compatibility tags exist.
- All committed changes pass `git diff --check`.
