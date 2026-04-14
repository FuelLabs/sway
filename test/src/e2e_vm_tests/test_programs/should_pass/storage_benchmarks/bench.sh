#!/usr/bin/env bash
#
# Runs storage benchmark projects and prints results as CSV and as
# a console histogram.
#
# Usage:
#   ./bench.sh [-h] [<project> ...]
#
# Options:
#   -h   Print a histogram alongside the CSV output.
#
# Examples:
#   ./bench.sh
#   ./bench.sh -h
#   ./bench.sh storage_fields
#   ./bench.sh -h storage_fields_partial_access
#
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/../../../../../.." && pwd)"

# ── Configurable constants ──────────────────────────────────────────
BAR_MAX_WIDTH=60          # max length of the histogram bar in characters
BAR_CHAR="█"
# ────────────────────────────────────────────────────────────────────

# ── Parse options ───────────────────────────────────────────────────
SHOW_HISTOGRAM=false
while getopts ":h" opt; do
    case $opt in
        h) SHOW_HISTOGRAM=true ;;
        *) echo "Usage: $0 [-h] [<project> ...]" >&2; exit 1 ;;
    esac
done
shift $((OPTIND - 1))

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
    local output
    output=$(cd "$REPO_ROOT" && cargo r -r -p forc -- test --release \
        -p "$project_path" 2>&1)

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
    local -a adjusted=()
    for i in "${!names[@]}"; do
        local adj=$(( gas_values[i] - baseline ))
        adjusted+=("$adj")
        echo "${names[$i]},$adj"
    done
    echo

    # ── Histogram (optional) ─────────────────────────────────────────
    if [[ "$SHOW_HISTOGRAM" == true ]]; then
        echo "--- Histogram (baseline $baseline gas subtracted) ---"
        echo

        # Strip "bench_" prefix from names for display to keep lines short.
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
            (( adjusted[i] > max_gas )) && max_gas=${adjusted[$i]}
            local glen=${#adjusted[$i]}
            (( glen > max_gas_len )) && max_gas_len=$glen
        done

        # Dynamically compute bar width to fit within the terminal.
        #   line layout:  "  " name " │ " bar " " gas_number
        #   overhead   :   2 + max_name_len + 3 + 1 + max_gas_len = 6 + max_name_len + max_gas_len
        local term_width
        term_width=$(tput cols 2>/dev/null || echo 120)
        local overhead=$(( 6 + max_name_len + max_gas_len ))
        local bar_max=$(( term_width - overhead ))
        (( bar_max < 10 )) && bar_max=10
        (( bar_max > BAR_MAX_WIDTH )) && bar_max=$BAR_MAX_WIDTH

        for i in "${!display_names[@]}"; do
            local gas=${adjusted[$i]}
            # Scale bar width.
            local bar_len=$(( gas * bar_max / max_gas ))
            # Ensure at least 1 char when gas > 0.
            (( gas > 0 && bar_len == 0 )) && bar_len=1

            local bar=""
            for (( b=0; b<bar_len; b++ )); do bar+="$BAR_CHAR"; done

            printf "  %-${max_name_len}s │ %s %${max_gas_len}s\n" "${display_names[$i]}" "$bar" "$gas"
        done

        echo
    fi
}

# ── Main ────────────────────────────────────────────────────────────

for project in "${PROJECTS[@]}"; do
    run_project "$project"
done
