# Sway + Rust test templates

This directory contains `cargo generate` templates for Rust integration tests:

| Template | Sway program type |
| --- | --- |
| `sway-test-rs` | Contract |
| `sway-script-test-rs` | Script |
| `sway-predicate-test-rs` | Predicate |

All three templates use the same Rust SDK minor as the `fuels` dependency in
the repository root and the SDK harness exercised by Sway CI. CI checks this
relationship so one program type cannot silently fall behind the others.

On `master`, the templates follow the default branch and may require unreleased
Sway behavior. A template from a `vX.Y.Z` tag is the release-specific snapshot;
generate from the tag that matches your Forc version rather than assuming the
default branch is compatible. Confirm the compiler with `forc --version`.
