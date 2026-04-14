#!/usr/bin/env bash
#
# Compares gas usage between two benchmark runs recorded in RESULTS.md.
#
# Usage:
#   ./perf_diff.sh <before-sha-substring> <after-sha-substring>
#
# The script searches RESULTS.md for commits whose full SHA contains
# the given substrings and produces two output files:
#
#   <last8-before> vs <last8-after>.perf-diff.csv
#   <last8-before> vs <last8-after>.perf-diff.md
#
# Both files contain: Project, Bench, Before, After, Difference, Percentage
#
#   Difference = -(After - Before)   (positive = improvement, negative = regression)
#   Percentage = relative change     (positive = improvement, negative = regression)
#
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
RESULTS_FILE="$SCRIPT_DIR/RESULTS.md"

# ── Argument validation ─────────────────────────────────────────────

if [[ $# -ne 2 ]]; then
    echo "Usage: $0 <before-sha-substring> <after-sha-substring>" >&2
    exit 1
fi

BEFORE_PATTERN="$1"
AFTER_PATTERN="$2"

if [[ -z "$BEFORE_PATTERN" || -z "$AFTER_PATTERN" ]]; then
    echo "ERROR: both SHA substrings must be non-empty." >&2
    exit 1
fi

if [[ "$BEFORE_PATTERN" == "$AFTER_PATTERN" ]]; then
    echo "ERROR: the two SHA substrings must be different." >&2
    exit 1
fi

if [[ ! -f "$RESULTS_FILE" ]]; then
    echo "ERROR: RESULTS.md not found at $RESULTS_FILE" >&2
    exit 1
fi

# ── Find matching commits ───────────────────────────────────────────

# Extract all full SHAs from the heading lines.
# Heading format: # Branch: `master` on 2026.04.08 `<full-sha>` (...)
mapfile -t ALL_SHAS < <(grep -oP '(?<=`)[0-9a-f]{40}(?=`)' "$RESULTS_FILE")

if [[ ${#ALL_SHAS[@]} -eq 0 ]]; then
    echo "ERROR: no commit SHAs found in RESULTS.md." >&2
    exit 1
fi

find_sha() {
    local pattern="$1"
    local label="$2"
    local -a matches=()

    for sha in "${ALL_SHAS[@]}"; do
        if [[ "$sha" == *"$pattern"* ]]; then
            matches+=("$sha")
        fi
    done

    if [[ ${#matches[@]} -eq 0 ]]; then
        echo "ERROR: no commit matching '$pattern' found in RESULTS.md." >&2
        echo "Available commits:" >&2
        printf "  %s\n" "${ALL_SHAS[@]}" >&2
        exit 1
    fi

    if [[ ${#matches[@]} -gt 1 ]]; then
        echo "ERROR: '$pattern' matches multiple commits ($label):" >&2
        printf "  %s\n" "${matches[@]}" >&2
        exit 1
    fi

    echo "${matches[0]}"
}

BEFORE_SHA=$(find_sha "$BEFORE_PATTERN" "before")
AFTER_SHA=$(find_sha "$AFTER_PATTERN" "after")

if [[ "$BEFORE_SHA" == "$AFTER_SHA" ]]; then
    echo "ERROR: both patterns resolve to the same commit: $BEFORE_SHA" >&2
    exit 1
fi

BEFORE_SHORT="${BEFORE_SHA: -8}"
AFTER_SHORT="${AFTER_SHA: -8}"

echo "Before: $BEFORE_SHA (…$BEFORE_SHORT)"
echo "After:  $AFTER_SHA (…$AFTER_SHORT)"
echo

# ── Parse benchmark data for a commit ───────────────────────────────

# parse_commit <sha>
# Outputs lines: project,bench_name,gas
parse_commit() {
    local sha="$1"
    local in_commit=false
    local in_csv=false
    local current_project=""

    while IFS= read -r line; do
        # Detect commit heading.
        if [[ "$line" =~ ^"# Branch:" ]]; then
            if [[ "$line" == *"$sha"* ]]; then
                in_commit=true
            elif $in_commit; then
                # We've moved past our commit's section.
                break
            fi
            continue
        fi

        $in_commit || continue

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
            if [[ -z "$line" || "$line" == '```' || "$line" == ---* || "$line" == ═* ]]; then
                in_csv=false
                continue
            fi
            # Output: project,bench_name,gas
            echo "${current_project},${line}"
        fi
    done < "$RESULTS_FILE"
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
done < <(parse_commit "$BEFORE_SHA")

while IFS=, read -r project bench gas; do
    local_key="${project},${bench}"
    AFTER_DATA["$local_key"]="$gas"
    if [[ -z "${SEEN_KEYS[$local_key]+x}" ]]; then
        KEYS+=("$local_key")
        SEEN_KEYS["$local_key"]=1
    fi
done < <(parse_commit "$AFTER_SHA")

if [[ ${#KEYS[@]} -eq 0 ]]; then
    echo "ERROR: no benchmark data found for the given commits." >&2
    exit 1
fi

# ── Compute diffs and generate output ───────────────────────────────

OUT_BASE="${BEFORE_SHORT} vs ${AFTER_SHORT}"
CSV_FILE="$SCRIPT_DIR/${OUT_BASE}.perf-diff.csv"
MD_FILE="$SCRIPT_DIR/${OUT_BASE}.perf-diff.md"

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
    echo "# Performance diff: …${BEFORE_SHORT} vs …${AFTER_SHORT}"
    echo
    echo "- **Before**: \`${BEFORE_SHA}\`"
    echo "- **After**:  \`${AFTER_SHA}\`"
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
