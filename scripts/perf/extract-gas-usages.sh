#!/bin/bash

# This script extracts full test names and test gas usages from a `forc test` output.
# The output is a CSV with two columns, test name, and gas usage.
# E.g.: `forc test | extract-gas-usages.sh`.
# If $1 is not empty it will be appended as a new column to the resulting CSV.

current_suite=""
results=()

while IFS= read -r line; do
    if [[ $line =~ ^tested\ --\ ([^[:space:]]+) ]]; then
        current_suite="${BASH_REMATCH[1]}"
    fi

    if [[ $line =~ ^[[:space:]]*test[[:space:]]([^\ ]+)[[:space:]]\.\.\.[[:space:]].*,[[:space:]]([0-9]+)[[:space:]]gas\) ]]; then
        test_name="${BASH_REMATCH[1]}"
        gas="${BASH_REMATCH[2]}"

        if [ "$1" = "" ]; then
            results+=("${current_suite}::${test_name},${gas}")
        else
            results+=("${current_suite}::${test_name},${gas},$1")
        fi
    fi
done

printf '%s\n' "${results[@]}" | sort
