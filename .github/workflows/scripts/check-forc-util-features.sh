#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/../../.."
manifest=forc-util/tests/features/Cargo.toml
# Reuse the workspace's dependency versions in the independent consumer workspace.
cp Cargo.lock forc-util/tests/features/Cargo.lock
export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-$PWD/target}/forc-util-features"

for feature in none bytecode cli diagnostics fs-locking restricted tx defaults all; do
    args=(--manifest-path "$manifest" --no-default-features)
    case "$feature" in
        none) ;;
        all) args+=(--all-features) ;;
        *) args+=(--features "$feature") ;;
    esac
    echo "Testing forc-util features: $feature"
    cargo test "${args[@]}"

    case "$feature" in
        none|bytecode|cli|fs-locking|restricted)
            tree=$(cargo tree "${args[@]}" --edges normal,build --prefix none)
            if grep -Eq '^(sway-core|sway-error|sway-types|fuel-tx|fuels-core) v' <<< "$tree"; then
                echo "Unexpected compiler or transaction dependency for $feature" >&2
                exit 1
            fi
            ;;
    esac
done
