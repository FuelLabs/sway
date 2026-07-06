#!/usr/bin/env bash
#
# Runs storage_vec benchmark projects and prints results as CSV.
# Each project benchmarks StorageVec operations for one element size.
#
# Baselines are per-count (not a single global baseline):
#   - bench_baseline_nN            → cost of populating N elements
#   - bench_baseline_store_vec_nN  → cost of building a heap Vec of N elements
#
# For "store_vec" tests the store_vec baseline is subtracted;
# for all other tests the populate baseline is subtracted.
#
# The output is printed to the console and also saved to a file in the
# `perf_out` folder, named:
#
#   MMDDHHMMSS-<branch name>.storage_vec.txt
#
# where <branch name> is the normalized name of the current git branch.
#
# If the BENCH_NO_SAVE environment variable is set (used by bench.sh),
# the output is only printed to the console and not saved to a file.
#
# Usage:
#   ./bench_storage_vec.sh [<project> ...]
#
# Examples:
#   ./bench_storage_vec.sh
#   ./bench_storage_vec.sh storage_vec_s8
#
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/../../../../../.." && pwd)"

ALL_PROJECTS=(
    storage_vec_s8
    storage_vec_s24
    storage_vec_s32
    storage_vec_s56
    storage_vec_s72
    storage_vec_s88
    storage_vec_s96
)

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
    echo "No StorageVec benchmark projects match the filter '${BENCH_FILTER:-}'." >&2
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
    # --no-gas-limit is needed because some O(N) operations on large structs
    # at n=1000 exceed the default gas limit and would otherwise revert.
    local output
    output=$(cd "$REPO_ROOT" && cargo r -r -p forc -- test --release \
        --no-gas-limit -p "$project_path")

    # Parse lines like:
    #       test bench_push_n100 ... ok (67.364µs, 12198 gas)
    local -a names=()
    local -a gas_values=()

    # Baselines keyed by count (N).
    local empty_call_baseline=0
    declare -A populate_baseline=()
    declare -A store_vec_baseline=()

    while IFS= read -r line; do
        local name gas
        name=$(echo "$line" | sed -E 's/.*test ([^ ]+) .*/\1/')
        gas=$(echo "$line"  | sed -E 's/.*, ([0-9]+) gas\).*/\1/')

        # Empty-call baseline: bench_baseline (no suffix)
        if [[ "$name" == "bench_baseline" ]]; then
            empty_call_baseline="$gas"
            continue
        fi

        # Populate baselines: bench_baseline_n<N>
        if [[ "$name" =~ ^bench_baseline_n([0-9]+)$ ]]; then
            populate_baseline["${BASH_REMATCH[1]}"]="$gas"
            continue
        fi

        # Store-vec baselines: bench_baseline_store_vec_n<N>
        if [[ "$name" =~ ^bench_baseline_store_vec_n([0-9]+)$ ]]; then
            store_vec_baseline["${BASH_REMATCH[1]}"]="$gas"
            continue
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
    echo "--- CSV (per-count baselines subtracted) ---"
    printf "  empty-call baseline: %s\n" "$empty_call_baseline"
    printf "  populate baselines : "
    for key in $(echo "${!populate_baseline[@]}" | tr ' ' '\n' | sort -n); do
        printf "n%s=%s  " "$key" "${populate_baseline[$key]}"
    done
    echo
    printf "  store_vec baselines: "
    for key in $(echo "${!store_vec_baseline[@]}" | tr ' ' '\n' | sort -n); do
        printf "n%s=%s  " "$key" "${store_vec_baseline[$key]}"
    done
    echo
    echo "test,gas"

    for i in "${!names[@]}"; do
        local test_name="${names[$i]}"
        local test_gas="${gas_values[$i]}"

        # Extract count N from the _nNNN suffix.
        local count
        count=$(echo "$test_name" | sed -E 's/.*_n([0-9]+)$/\1/')

        # Extract operation name (strip bench_ prefix and _nN suffix).
        local op
        op=$(echo "$test_name" | sed -E 's/^bench_//; s/_n[0-9]+$//')

        # Pick the right baseline.
        #   push_n_elems_into_empty_vec  → empty-call baseline (measures the full push loop)
        #   store_vec                    → store_vec baseline  (build heap Vec only)
        #   everything                   → populate baseline   (push N elements setup)
        local baseline=0
        if [[ "$op" == "push_n_elems_into_empty_vec" ]]; then
            baseline="$empty_call_baseline"
        elif [[ "$op" == "store_vec" ]]; then
            baseline="${store_vec_baseline[$count]:-0}"
        else
            baseline="${populate_baseline[$count]:-0}"
        fi

        local adj=$(( test_gas - baseline ))
        echo "${test_name},$adj"
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
    OUT_FILE="$OUT_DIR/${TIMESTAMP}-${BRANCH}.storage_vec.txt"

    for project in "${PROJECTS[@]}"; do
        run_project "$project"
    done | tee "$OUT_FILE"

    echo "Results saved to: $OUT_FILE"
fi
