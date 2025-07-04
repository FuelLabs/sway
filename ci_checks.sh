#!/usr/bin/env bash

# The script runs almost all CI checks locally.
#
# Tests below requires running `fuel-core` node locally:
# - `cargo run --locked --release --bin test -- --locked`
#
# You can install `fuel-core` node by:
# `cargo install fuel-core-bin --git https://github.com/FuelLabs/fuel-core --tag v0.24.3 --locked`
#
# And run it with:
# `fuel-core run --db-type in-memory --debug --snapshot ./.github/workflows/local-testnode`

# Requires installed:
# `cargo install cargo-sort`
# `cargo install cargo-generate`
# `cargo install cargo-udeps`

./.github/workflows/scripts/check-sdk-harness-version.sh
cargo clippy --all-features --all-targets -- -D warnings &&
cargo sort -w --check &&
cargo sort -w --check templates/sway-test-rs/template &&
cargo fmt --all -- --check &&
cargo build --locked --workspace --all-features --all-targets &&
cargo test --locked &&
cargo +nightly udeps --locked --all-targets &&
cargo install --locked --debug --path ./forc &&
cargo install --locked --debug --path ./forc-plugins/forc-fmt &&
cargo install --locked --debug --path ./forc-plugins/forc-lsp &&
cargo install --locked --debug --path ./forc-plugins/forc-client &&
cargo install --locked --debug --path ./forc-plugins/forc-tx &&
cargo install --locked --debug --path ./scripts/mdbook-forc-documenter &&
forc build --path sway-lib-std &&
forc test --path sway-lib-std &&
cargo run --locked -p forc -- build --locked --path ./examples/Forc.toml &&
cargo run --locked -p forc-fmt -- --check --path ./examples &&
cargo run --locked -p forc -- build --path ./docs/reference/src/code/Forc.toml &&
rm -Rf test-proj &&
forc new test-proj &&
echo "std = { path = \"../sway-lib-std/\" }" >> test-proj/Forc.toml &&
forc build --path test-proj &&
(cd test-proj && cargo generate --init --path ../templates/sway-test-rs --name test-proj) &&
echo "[workspace]" >> test-proj/Cargo.toml &&
(cd test-proj && cargo test) &&
rm -R test-proj &&
cargo run --locked --release --bin test -- --target evm --locked &&
cargo run --locked -p forc -- build --locked --path ./test/src/sdk-harness &&
cargo test --locked --manifest-path ./test/src/sdk-harness/Cargo.toml -- --nocapture &&
cargo run --locked --release --bin test -- --locked
