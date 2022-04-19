#!/bin/bash
cargo fmt --all -- --check &&
git ls-files | grep Cargo.toml$ | xargs --verbose -n 1 cargo-toml-lint &&
cargo clippy --all-features --all-targets -- -D warnings