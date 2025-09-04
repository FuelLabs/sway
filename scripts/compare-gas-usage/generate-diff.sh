#!/bin/bash

# This script compares test gas usage from two outputs of the `test_gas_usage.sh` script.
# The result of the comparison can be printed either as a Markdown table or a CSV file.
# Usage: `compare_test_gas_usage.sh <before>.csv <after>.csv [MD|CSV]`.

if [ "$#" -lt 2 ] || [ "$#" -gt 3 ]; then
  echo "Usage: $0 <before>.csv <after>.csv [MD|CSV]"
  exit 1
fi

before_file="$1"
after_file="$2"
output_format="${3:-MD}"

if [ "$output_format" == "CSV" ]; then
  echo "Test,Before,After,Percentage"
else
  echo "| Test | Before | After | Percentage |"
  echo "|------|--------|-------|------------|"
fi

paste -d, "$before_file" "$after_file" | while IFS=',' read -r test1 before test2 after; do
  if [ "$before" != "$after" ]; then
    diff=$((before - after))
    percent=$(LC_NUMERIC=C awk -v d="$diff" -v b="$before" 'BEGIN { printf "%.2f", (d / b) * 100 }')

    if [ "$output_format" == "CSV" ]; then
      echo "$test1,$before,$after,$percent"
    else
      echo "| $test1 | $before | $after | $percent% |"
    fi
  fi
done