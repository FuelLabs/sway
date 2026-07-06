#!/usr/bin/env bash
#
# Compares gas usage between two benchmark runs saved in the `perf_out`
# folder by bench.sh or bench_storage_vec.sh.
#
# Usage:
#   ./perf_diff.sh <before-file> <after-file>
#
# Example:
#   ./perf_diff.sh perf_out/0611120000-master.storage_vec.txt \
#                  perf_out/0611140000-my_branch.storage_vec.txt
#
# Both files must be of the same kind (fields, storage_vec, or all);
# mixing kinds is an error.
#
# Produces two output files in the `perf_out` folder:
#
#   <before-timestamp> vs <after-timestamp>.perf-diff.csv
#   <before-timestamp> vs <after-timestamp>.perf-diff.md
#
# Both files contain: Project, Bench, Before, After, Difference, Percentage
#
#   Difference = -(After - Before)   (positive = improvement, negative = regression)
#   Percentage = relative change     (positive = improvement, negative = regression)
#
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
OUT_DIR="$SCRIPT_DIR/perf_out"

# ── Argument validation ─────────────────────────────────────────────

if [[ $# -ne 2 ]]; then
    echo "Usage: $0 <before-file> <after-file>" >&2
    exit 1
fi

BEFORE_FILE="$1"
AFTER_FILE="$2"

for f in "$BEFORE_FILE" "$AFTER_FILE"; do
    if [[ ! -f "$f" ]]; then
        echo "ERROR: file not found: $f" >&2
        exit 1
    fi
done

if [[ "$(realpath "$BEFORE_FILE")" == "$(realpath "$AFTER_FILE")" ]]; then
    echo "ERROR: the two files must be different." >&2
    exit 1
fi

# Extract the timestamp and kind (fields|storage_vec|all) from a perf
# output file name of the form: MMDDHHMMSS-<branch>.<kind>.txt
# Sets PARSED_TIMESTAMP and PARSED_KIND.
parse_file_name() {
    local file="$1"
    local label="$2"
    local base
    base="$(basename "$file")"
    if [[ ! "$base" =~ ^([0-9]{10})-.+\.(fields|storage_vec|all)\.txt$ ]]; then
        echo "ERROR: $label file name does not match the scheme MMDDHHMMSS-<branch>.<fields|storage_vec|all>.txt: $base" >&2
        exit 1
    fi
    PARSED_TIMESTAMP="${BASH_REMATCH[1]}"
    PARSED_KIND="${BASH_REMATCH[2]}"
}

parse_file_name "$BEFORE_FILE" "before"
BEFORE_TIMESTAMP="$PARSED_TIMESTAMP"
BEFORE_KIND="$PARSED_KIND"

parse_file_name "$AFTER_FILE" "after"
AFTER_TIMESTAMP="$PARSED_TIMESTAMP"
AFTER_KIND="$PARSED_KIND"

if [[ "$BEFORE_KIND" != "$AFTER_KIND" ]]; then
    echo "ERROR: cannot compare different benchmark kinds: $BEFORE_KIND vs $AFTER_KIND." >&2
    exit 1
fi

# Strip directory and .txt extension to get the run name.
BEFORE_NAME="$(basename "$BEFORE_FILE" .txt)"
AFTER_NAME="$(basename "$AFTER_FILE" .txt)"

echo "Before: $BEFORE_FILE"
echo "After:  $AFTER_FILE"
echo

# ── Parse benchmark data from a perf output file ────────────────────

# parse_perf_file <file>
# Outputs lines: project,bench_name,gas
parse_perf_file() {
    local file="$1"
    local in_csv=false
    local current_project=""

    while IFS= read -r line; do
        # Detect project name from "Running: <project>" lines.
        if [[ "$line" =~ Running:[[:space:]]+(.+) ]]; then
            current_project="${BASH_REMATCH[1]}"
            current_project="${current_project%"${current_project##*[![:space:]]}"}"  # trim trailing whitespace
            in_csv=false
            continue
        fi

        # Detect start of CSV data (the "test,gas" header).
        if [[ "$line" == "test,gas" ]]; then
            in_csv=true
            continue
        fi

        # End of CSV block.
        if $in_csv; then
            if [[ -z "$line" || "$line" == ---* || "$line" == ═* ]]; then
                in_csv=false
                continue
            fi
            # Output: project,bench_name,gas
            echo "${current_project},${line}"
        fi
    done < "$file"
}

# ── Collect data ────────────────────────────────────────────────────

declare -A BEFORE_DATA=()
declare -A AFTER_DATA=()
declare -a KEYS=()        # ordered list of "project,bench" keys
declare -A SEEN_KEYS=()

while IFS=, read -r project bench gas; do
    local_key="${project},${bench}"
    BEFORE_DATA["$local_key"]="$gas"
    if [[ -z "${SEEN_KEYS[$local_key]+x}" ]]; then
        KEYS+=("$local_key")
        SEEN_KEYS["$local_key"]=1
    fi
done < <(parse_perf_file "$BEFORE_FILE")

while IFS=, read -r project bench gas; do
    local_key="${project},${bench}"
    AFTER_DATA["$local_key"]="$gas"
    if [[ -z "${SEEN_KEYS[$local_key]+x}" ]]; then
        KEYS+=("$local_key")
        SEEN_KEYS["$local_key"]=1
    fi
done < <(parse_perf_file "$AFTER_FILE")

if [[ ${#KEYS[@]} -eq 0 ]]; then
    echo "ERROR: no benchmark data found in the given files." >&2
    exit 1
fi

# ── Compute diffs and generate output ───────────────────────────────

mkdir -p "$OUT_DIR"
OUT_BASE="${BEFORE_TIMESTAMP} vs ${AFTER_TIMESTAMP}"
CSV_FILE="$OUT_DIR/${OUT_BASE}.perf-diff.csv"
MD_FILE="$OUT_DIR/${OUT_BASE}.perf-diff.md"

# CSV output
{
    echo "Project,Bench,Before,After,Difference,Percentage"
    for key in "${KEYS[@]}"; do
        IFS=, read -r project bench <<< "$key"
        before="${BEFORE_DATA[$key]:-}"
        after="${AFTER_DATA[$key]:-}"

        if [[ -z "$before" || -z "$after" ]]; then
            diff="N/A"
            pct="N/A"
            [[ -z "$before" ]] && before="N/A"
            [[ -z "$after" ]] && after="N/A"
        else
            # Difference = -(After - Before)
            diff=$(( -(after - before) ))
            if [[ "$before" -eq 0 ]]; then
                pct="N/A"
            else
                # Percentage with two decimal places: diff / before * 100
                # Using awk for floating point. LC_NUMERIC=C forces dot as decimal separator.
                pct=$(LC_NUMERIC=C awk "BEGIN { printf \"%.2f\", ($diff / $before) * 100 }")
                pct="${pct}%"
            fi
        fi
        echo "${project},${bench},${before},${after},${diff},${pct}"
    done
} > "$CSV_FILE"

# MD output
{
    echo "# Performance diff: ${BEFORE_NAME} vs ${AFTER_NAME}"
    echo
    echo "- **Before**: \`${BEFORE_NAME}\`"
    echo "- **After**:  \`${AFTER_NAME}\`"
    echo
    echo "Difference = -(After - Before). Positive = improvement, negative = regression."
    echo

    # Compute column widths for nice alignment.
    # Fixed headers.
    local_headers=("Project" "Bench" "Before" "After" "Difference" "Percentage")
    declare -a col_w=()
    for h in "${local_headers[@]}"; do
        col_w+=("${#h}")
    done

    # Collect all rows first to measure widths.
    declare -a rows=()
    for key in "${KEYS[@]}"; do
        IFS=, read -r project bench <<< "$key"
        before="${BEFORE_DATA[$key]:-}"
        after="${AFTER_DATA[$key]:-}"

        if [[ -z "$before" || -z "$after" ]]; then
            diff="N/A"
            pct="N/A"
            [[ -z "$before" ]] && before="N/A"
            [[ -z "$after" ]] && after="N/A"
        else
            diff=$(( -(after - before) ))
            if [[ "$before" -eq 0 ]]; then
                pct="N/A"
            else
                pct=$(LC_NUMERIC=C awk "BEGIN { printf \"%.2f\", ($diff / $before) * 100 }")
                pct="${pct}%"
            fi
        fi

        row="${project}|${bench}|${before}|${after}|${diff}|${pct}"
        rows+=("$row")

        # Update column widths.
        local_vals=("$project" "$bench" "$before" "$after" "$diff" "$pct")
        for c in "${!local_vals[@]}"; do
            local_len=${#local_vals[$c]}
            (( local_len > col_w[c] )) && col_w[$c]=$local_len
        done
    done

    # Print header.
    printf "| %-${col_w[0]}s | %-${col_w[1]}s | %${col_w[2]}s | %${col_w[3]}s | %${col_w[4]}s | %${col_w[5]}s |\n" \
        "${local_headers[0]}" "${local_headers[1]}" "${local_headers[2]}" "${local_headers[3]}" "${local_headers[4]}" "${local_headers[5]}"

    # Print separator: left-align for Project and Bench, right-align for numbers.
    sep_left() { local w=$1; printf ":"; printf -- '-%.0s' $(seq 1 "$w"); printf "-"; }
    sep_right() { local w=$1; printf "-"; printf -- '-%.0s' $(seq 1 "$w"); printf ":"; }
    printf "|%s|%s|%s|%s|%s|%s|\n" \
        "$(sep_left "${col_w[0]}")" \
        "$(sep_left "${col_w[1]}")" \
        "$(sep_right "${col_w[2]}")" \
        "$(sep_right "${col_w[3]}")" \
        "$(sep_right "${col_w[4]}")" \
        "$(sep_right "${col_w[5]}")"

    # Print data rows.
    for row in "${rows[@]}"; do
        IFS='|' read -r project bench before after diff pct <<< "$row"
        printf "| %-${col_w[0]}s | %-${col_w[1]}s | %${col_w[2]}s | %${col_w[3]}s | %${col_w[4]}s | %${col_w[5]}s |\n" \
            "$project" "$bench" "$before" "$after" "$diff" "$pct"
    done
} > "$MD_FILE"

echo "Generated:"
echo "  $CSV_FILE"
echo "  $MD_FILE"

# Print summary statistics.
total=0
improved=0
regressed=0
unchanged=0
skipped=0
for key in "${KEYS[@]}"; do
    before="${BEFORE_DATA[$key]:-}"
    after="${AFTER_DATA[$key]:-}"
    total=$(( total + 1 ))
    if [[ -z "$before" || -z "$after" ]]; then
        skipped=$(( skipped + 1 ))
        continue
    fi
    diff=$(( -(after - before) ))
    if (( diff > 0 )); then
        improved=$(( improved + 1 ))
    elif (( diff < 0 )); then
        regressed=$(( regressed + 1 ))
    else
        unchanged=$(( unchanged + 1 ))
    fi
done

echo
echo "Summary: $total benchmarks — $improved improved, $regressed regressed, $unchanged unchanged, $skipped skipped (missing data)"
