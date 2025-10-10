#!/bin/bash

# This script extracts full test names and test gas usage from a `forc test` output.
# Usage: `forc test | test_gas_usage.sh`.
# if $1 is not empty it will be appended as a new column to the csv

current_suite=""
results=()

while IFS= read -r line; do
    # printf 'Line: %s\n' "$line"

    if [[ $line =~ ^tested\ --\ ([^[:space:]]+) ]]; then
        current_suite="${BASH_REMATCH[1]}"
    fi
    # printf 'Suite: %s\n' "$current_suite"

    if [[ $line =~ ^[[:space:]]*test[[:space:]]([^\ ]+)[[:space:]]\.\.\.[[:space:]].*,[[:space:]]([0-9]+)[[:space:]]gas\) ]]; then
        test_name="${BASH_REMATCH[1]}"
        # printf 'Test: %s\n' "$test_name"
        gas="${BASH_REMATCH[2]}"
        # printf 'Gas: %s\n' "$gas"

        if [ "$1" = "" ]; then
            results+=("${current_suite}::${test_name},${gas}")
        else
            results+=("${current_suite}::${test_name},${gas},$1")
        fi
    fi
done

printf '%s\n' "${results[@]}" | sort