#!/usr/bin/env bash

# The script runs almost all CI checks locally.
#
# Tests below requires running `fuel-core` node locally:
# - `cargo run --locked --release --bin test -- --locked`
#
# You can install `fuel-core` node by:
# `cargo install fuel-core-bin --git https://github.com/FuelLabs/fuel-core --tag v0.16.1 --locked`
#
# And run it with:
# `fuel-core run --db-type in-memory`

# Requires installed:
# `cargo install cargo-sort`
# `cargo install cargo-generate`
# `cargo install cargo-udeps`

cargo clippy --all-features --all-targets &&
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
cargo install --locked --debug forc-explore &&
cargo install --locked --debug --path ./scripts/mdbook-forc-documenter &&
forc build --path sway-lib-core &&
forc test --path sway-lib-core &&
forc build --path sway-lib-std &&
forc test --path sway-lib-std &&
cargo run --locked --bin examples-checker build --all-examples &&
cargo run --locked --bin examples-checker fmt --all-examples &&
rm -Rf test-proj &&
forc new test-proj &&
echo "std = { path = \"../sway-lib-std/\" }" >> test-proj/Forc.toml &&
forc build --path test-proj &&
(cd test-proj && cargo generate --init --path ../templates/sway-test-rs --name test-proj) &&
echo "[workspace]" >> test-proj/Cargo.toml &&
(cd test-proj && cargo test) &&
rm -R test-proj &&
cargo run --locked --release --bin test -- --target evm --locked &&
(cd test/src/sdk-harness && bash build.sh --locked) &&
cargo test --manifest-path ./test/src/sdk-harness/Cargo.toml -- --nocapture &&
cargo run --locked --release --bin test -- --locked
