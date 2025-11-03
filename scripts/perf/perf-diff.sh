#!/bin/bash

# This script compares performance data (gas usages and bytecode sizes) from two CSV files.
# CSV files must have two columns, the test name and the performance data, and the test
# names must be the same and in the same order in both files.
# The result of the comparison can be printed either as a Markdown table or a CSV file.
# Usage: `perf-diff.sh <before>.csv <after>.csv [md|csv]`.

if [ "$#" -lt 2 ] || [ "$#" -gt 3 ]; then
    echo "Usage: $0 <before>.csv <after>.csv [md|csv]"
    exit 1
fi

before_file="$1"
after_file="$2"
output_format="${3:-md}"

if [[ "$output_format" != "md" && "$output_format" != "csv" ]]; then
    echo "ERROR: Invalid output format '$output_format'. Output format must be either 'md' or 'csv'."
    exit 2
fi

# Validate test name order and equality.
# We trim leading/trailing whitespace on the first column before comparing.
tmpdiff="$(mktemp)"
if ! diff -u \
    --label "names before ($before_file)" --label "names after ($after_file)" \
    <(awk -F',' '{n=$1; sub(/^[ \t]+/,"",n); sub(/[ \t]+$/,"",n); print n}' "$before_file") \
    <(awk -F',' '{n=$1; sub(/^[ \t]+/,"",n); sub(/[ \t]+$/,"",n); print n}' "$after_file") \
    >"$tmpdiff"; then
    echo "ERROR: Test names differ between:"
    echo " before: $before_file"
    echo "  and"
    echo " after:  $after_file"
    echo " files."
    echo "Both files must have the same tests in the same order."
    echo
    cat "$tmpdiff"
    rm -f "$tmpdiff"
    exit 3
fi
rm -f "$tmpdiff"

if [ "$output_format" == "csv" ]; then
    echo "Test,Before,After,Percentage"
else
    echo "| Test | Before | After | Percentage |"
    echo "|-----:|-------:|------:|-----------:|"
fi

paste -d, "$before_file" "$after_file" | while IFS=',' read -r test1 before test2 after; do
    if [ "$before" != "$after" ]; then
        diff=$((before - after))
        if [ "$before" -eq 0 ] 2>/dev/null; then
            percent="NaN"
        else
            percent=$(LC_NUMERIC=C awk -v d="$diff" -v b="$before" 'BEGIN { printf "%.2f", (d / b) * 100 }')
        fi

        if [ "$output_format" == "csv" ]; then
            echo "$test1,$before,$after,$percent"
        else
            echo "| $test1 | $before | $after | ${percent}% |"
        fi
    fi
done
