#!/bin/bash

# This script finds the latest two performance CSV files for a given performance category,
# e.g., `e2e-gas-usages`, in `./test/perf_out`, orders them by their timestamp prefix,
# and calls `perf-diff.sh` to compare them, with before being the older and after the newer file.
# Usage: perf-diff-latest.sh [md|csv]

output_format="${1:-md}"
perf_out_dir="./test/perf_out"
perf_diff_script="./scripts/perf/perf-diff.sh"

categories=("e2e-gas-usages" "e2e-bytecode-sizes" "in-language-gas-usages" "in-language-bytecode-sizes")

err() {
    echo "ERROR: $*" >&2
    exit 1
}

if [[ "$output_format" != "md" && "$output_format" != "csv" ]]; then
    err "Invalid output format '$output_format'.  Output format must be either 'md' or 'csv'."
fi

[[ -d "$perf_out_dir" ]] || err "Directory not found: '$perf_out_dir'."
command -v "$perf_diff_script" >/dev/null 2>&1 || [[ -x "$perf_diff_script" ]] || err "Cannot find performance diff script at: '$perf_diff_script'."

process_category() {
    local category="$1"

    # Collect candidates: "*-<category>-*.csv", but skip "-<category>-historical-*.csv".
    mapfile -t candidates < <(
        find "$perf_out_dir" -maxdepth 1 -type f -name "*-${category}-*.csv" -printf "%p\n" 2>/dev/null |
            while IFS= read -r f; do
                base="$(basename -- "$f")"

                # Skip historical comparisons and diffs.
                if [[ "$base" == *"-${category}-historical-"* || "$base" == *"-${category}-historical.csv" || "$base" == *"-diff-${category}-"* ]]; then
                    continue
                fi

                ts="${base%%-*}"
                [[ "$ts" =~ ^[0-9]+$ ]] || continue
                printf "%s\t%s\n" "$ts" "$f"
            done
    )

    if ((${#candidates[@]} == 0)); then
        echo "INFO: No files to compare for category '$category'."
        return 0
    fi

    # Sort by timestamp (asc) and take last two paths.
    mapfile -t sorted_paths < <(
        printf "%s\n" "${candidates[@]}" |
            sort -n -k1,1 |
            awk -F'\t' '{print $2}' |
            tail -n 2
    )

    if ((${#sorted_paths[@]} < 2)); then
        echo "INFO: No files to compare for category '$category' (found only one candidate)."
        return 0
    fi

    before_file="${sorted_paths[0]}"
    after_file="${sorted_paths[1]}"

    before_base="$(basename "$before_file")"
    after_base="$(basename "$after_file")"
    before_ts="${before_base%%-*}"
    after_ts="${after_base%%-*}"
    now_ts="$(date '+%m%d%H%M%S')"

    outfile="${perf_out_dir}/${now_ts}-diff-${category}-${before_ts}-vs-${after_ts}.${output_format}"

    echo "Comparing '$category':"
    echo " before: $before_file"
    echo " after:  $after_file"
    echo "Diff written to: $outfile"
    echo

    "$perf_diff_script" "$before_file" "$after_file" "$output_format" | tee "$outfile"
    echo
}

for cat in "${categories[@]}"; do
    process_category "$cat"
done
