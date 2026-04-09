#!/usr/bin/env bash
#
# Runs storage_vec benchmark projects and prints results as CSV and as
# a console histogram.  Each project benchmarks StorageVec operations
# for one element size.
#
# Baselines are per-count (not a single global baseline):
#   - bench_baseline_nN            → cost of populating N elements
#   - bench_baseline_store_vec_nN  → cost of building a heap Vec of N elements
#
# For "store_vec" tests the store_vec baseline is subtracted;
# for all other tests the populate baseline is subtracted.
#
# Usage:
#   ./bench_storage_vec.sh                   # run all sizes
#   ./bench_storage_vec.sh storage_vec_s8    # run a single size
#
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/../../../../../.." && pwd)"

# ── Configurable constants ──────────────────────────────────────────
BAR_MAX_WIDTH=60
BAR_CHAR="█"
# ────────────────────────────────────────────────────────────────────

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

    # Run forc test and capture output.
    # --no-gas-limit is needed because some O(N) operations on large structs
    # at n=1000 exceed the default gas limit and would otherwise revert.
    local output
    output=$(cd "$REPO_ROOT" && cargo r -r -p forc -- test --release \
        --no-gas-limit -p "$project_path" 2>&1)

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

    local -a adjusted=()
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
        adjusted+=("$adj")
        echo "${test_name},$adj"
    done
    echo

    # ── Histogram ───────────────────────────────────────────────────
    echo "--- Histogram ---"
    echo

    # Strip bench_ prefix for display.
    local -a display_names=()
    for i in "${!names[@]}"; do
        display_names+=("${names[$i]#bench_}")
    done

    # Find the longest display name and the maximum gas value for scaling.
    local max_name_len=0
    local max_gas=1
    local max_gas_len=1
    for i in "${!display_names[@]}"; do
        local nlen=${#display_names[$i]}
        (( nlen > max_name_len )) && max_name_len=$nlen
        local abs_gas=${adjusted[$i]}
        (( abs_gas < 0 )) && abs_gas=$(( -abs_gas ))
        (( abs_gas > max_gas )) && max_gas=$abs_gas
        local glen=${#adjusted[$i]}
        (( glen > max_gas_len )) && max_gas_len=$glen
    done

    # Dynamically compute bar width to fit within the terminal.
    local term_width
    term_width=$(tput cols 2>/dev/null || echo 120)
    local overhead=$(( 6 + max_name_len + max_gas_len ))
    local bar_max=$(( term_width - overhead ))
    (( bar_max < 10 )) && bar_max=10
    (( bar_max > BAR_MAX_WIDTH )) && bar_max=$BAR_MAX_WIDTH

    for i in "${!display_names[@]}"; do
        local gas=${adjusted[$i]}
        local abs_gas=$gas
        (( abs_gas < 0 )) && abs_gas=0
        # Scale bar width.
        local bar_len=$(( abs_gas * bar_max / max_gas ))
        # Ensure at least 1 char when gas > 0.
        (( abs_gas > 0 && bar_len == 0 )) && bar_len=1

        local bar=""
        for (( b=0; b<bar_len; b++ )); do bar+="$BAR_CHAR"; done

        printf "  %-${max_name_len}s │ %s %${max_gas_len}s\n" "${display_names[$i]}" "$bar" "$gas"
    done

    echo
}

# ── Main ────────────────────────────────────────────────────────────

for project in "${PROJECTS[@]}"; do
    run_project "$project"
done
