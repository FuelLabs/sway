#!/usr/bin/env bash

# Run `forc test` individually for each project under test_programs/.
#
# Usage: ./run_in_language_tests.sh [extra `forc test` args, e.g. --release --experimental ...]
#
# Note: --error-on-warnings arg is always passed to `forc test`.
#
# The script continues even when individual test projects fail, and exits with
# a non-zero code at the end if any project failed.

# TODO: This is a workaround for the issue of `forc test` process getting killed
#       when running tests on a workspace with large number of projects.
#       Remove this file and switch to using `forc test` on the entire workspace
#       once https://github.com/FuelLabs/sway/issues/7613 is resolved.

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
TEST_PROGRAMS_DIR="$SCRIPT_DIR/test_programs"

# Always use the `forc` binary built in the repository root (3 levels above here:
# test/src/in_language_tests/ -> sway/).
REPO_ROOT="$(cd "$SCRIPT_DIR/../../.." && pwd)"
FORC="$REPO_ROOT/target/release/forc"
if [[ ! -x "$FORC" ]]; then
    echo "Error: \`forc\` binary not found at $FORC" >&2
    echo "Build it with: cargo build --release -p forc" >&2
    exit 1
fi

# Projects under test_programs/ that are not test projects and should be skipped.
EXCLUDED_PROJECTS=(
    "test_types"
)

failed=()
passed=()

while IFS= read -r -d '' forc_toml; do
    project_dir="$(dirname "$forc_toml")"
    project_name="$(basename "$project_dir")"

    if [[ " ${EXCLUDED_PROJECTS[*]} " == *" ${project_name} "* ]]; then
        continue
    fi

    echo ""
    echo "==> Testing: $project_name"

    if "$FORC" test --error-on-warnings "$@" --path "$project_dir"; then
        passed+=("$project_name")
    else
        echo "FAILED: $project_name" >&2
        failed+=("$project_name")
    fi
done < <(find "$TEST_PROGRAMS_DIR" -mindepth 2 -maxdepth 2 -name "Forc.toml" -print0 | sort -z)

echo ""
echo "================================================"
echo "Results: ${#passed[@]} passed, ${#failed[@]} failed"
echo "================================================"

if [[ ${#failed[@]} -gt 0 ]]; then
    echo ""
    echo "Failed tests (${#failed[@]}):"
    for f in "${failed[@]}"; do
        echo "  - $f"
    done
    exit 1
fi

echo "All ${#passed[@]} tests passed!"
