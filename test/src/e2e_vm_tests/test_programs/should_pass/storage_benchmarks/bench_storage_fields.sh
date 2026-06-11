#!/usr/bin/env bash
#
# Runs storage benchmark projects and prints results as CSV.
#
# The output is printed to the console and also saved to a file in the
# `perf_out` folder, named:
#
#   MMDDHHMMSS-<branch name>.fields.txt
#
# where <branch name> is the normalized name of the current git branch.
#
# If the BENCH_NO_SAVE environment variable is set (used by bench.sh),
# the output is only printed to the console and not saved to a file.
#
# Usage:
#   ./bench_storage_fields.sh [<project> ...]
#
# Examples:
#   ./bench_storage_fields.sh
#   ./bench_storage_fields.sh storage_fields
#
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/../../../../../.." && pwd)"

# All benchmark project directories (order matters for output).
ALL_PROJECTS=(
    storage_fields
    storage_fields_partial_access
)

# If the user passed project names on the CLI, use those instead.
if [[ $# -gt 0 ]]; then
    PROJECTS=("$@")
else
    PROJECTS=("${ALL_PROJECTS[@]}")
fi

# If BENCH_FILTER is set (used by bench.sh), keep only the projects
# whose names contain the filter as a substring.
if [[ -n "${BENCH_FILTER:-}" ]]; then
    FILTERED=()
    for p in "${PROJECTS[@]}"; do
        [[ "$p" == *"${BENCH_FILTER}"* ]] && FILTERED+=("$p")
    done
    PROJECTS=()
    if [[ ${#FILTERED[@]} -gt 0 ]]; then
        PROJECTS=("${FILTERED[@]}")
    fi
fi

if [[ ${#PROJECTS[@]} -eq 0 ]]; then
    echo "No storage field benchmark projects match the filter '${BENCH_FILTER:-}'." >&2
    exit 0
fi

# ── Helpers ─────────────────────────────────────────────────────────

run_project() {
    local project="$1"
    local project_path="$SCRIPT_DIR/$project"

    if [[ ! -d "$project_path" ]]; then
        echo "ERROR: project directory not found: $project_path" >&2
        return 1
    fi

    echo "═══════════════════════════════════════════════════════════════"
    echo "  Running: $project"
    echo "═══════════════════════════════════════════════════════════════"
    echo

    # Run forc test and capture its stdout (the test result lines).
    # stderr is left untouched so that cargo build progress and forc
    # warnings stream live to the console (and stay out of the saved
    # perf_out file).
    local output
    output=$(cd "$REPO_ROOT" && cargo r -r -p forc -- test --release \
        -p "$project_path")

    # Parse lines like:
    #       test bench_baseline ... ok (67.364µs, 12198 gas)
    # into pairs: test_name  gas_value
    local -a names=()
    local -a gas_values=()
    local baseline=0

    while IFS= read -r line; do
        # Extract test name and gas.
        local name gas
        name=$(echo "$line" | sed -E 's/.*test ([^ ]+) .*/\1/')
        gas=$(echo "$line"  | sed -E 's/.*, ([0-9]+) gas\).*/\1/')

        if [[ "$name" == "bench_baseline" ]]; then
            baseline="$gas"
            continue            # baseline itself is not included in the results
        fi

        names+=("$name")
        gas_values+=("$gas")
    done < <(echo "$output" | grep -E '^\s+test bench_')

    if [[ ${#names[@]} -eq 0 ]]; then
        echo "WARNING: no benchmark results found for $project" >&2
        echo
        return
    fi

    # ── CSV output ──────────────────────────────────────────────────
    echo "--- CSV (baseline $baseline gas subtracted) ---"
    echo "test,gas"
    for i in "${!names[@]}"; do
        local adj=$(( gas_values[i] - baseline ))
        echo "${names[$i]},$adj"
    done
    echo
}

# ── Main ────────────────────────────────────────────────────────────

if [[ -n "${BENCH_NO_SAVE:-}" ]]; then
    for project in "${PROJECTS[@]}"; do
        run_project "$project"
    done
else
    OUT_DIR="$SCRIPT_DIR/perf_out"
    mkdir -p "$OUT_DIR"

    TIMESTAMP=$(date +%m%d%H%M%S)
    BRANCH=$(git -C "$SCRIPT_DIR" rev-parse --abbrev-ref HEAD | sed 's![^A-Za-z0-9._-]!_!g')
    OUT_FILE="$OUT_DIR/${TIMESTAMP}-${BRANCH}.fields.txt"

    for project in "${PROJECTS[@]}"; do
        run_project "$project"
    done | tee "$OUT_FILE"

    echo "Results saved to: $OUT_FILE"
fi
