#!/bin/bash

# This script extracts project names and bytecode sizes from a `forc test` or
# `forc build` output.
# The output is a CSV with two columns, project name, and bytecode size in bytes.
# E.g.: `forc test --release | extract-bytecode-sizes.sh`.
# If $1 is not empty it will be appended as a new column to the resulting CSV.
#
# Only sizes from RELEASE builds are extracted, i.e. from lines of the form:
#   Finished release [optimized + fuel] target(s) [<size>] in ???
# (debug builds, `Finished debug ...`, are ignored).
#
# The project name for a size is resolved as:
#   - the project from the preceding `tested -- <project>` line, which
#     `run_in_language_tests.sh` emits for each project (`forc test` output), or
#   - if there is none, the project from the last `Compiling <type> <project> ...`
#     line before the `Finished` line (`forc build` output).
#
# Note on sizes: `forc` pretty-prints the size using DECIMAL (base-1000) units, so
# the printed value is misleading if interpreted as base-1024. For example,
# `[21.68 KB]` is exactly 21680 bytes (21.68 * 1000), NOT 21.68 * 1024. This script
# converts the printed value back to the exact byte count.

esc=$'\e'
ansi_re="$esc\[[0-9;]*[a-zA-Z]"

tested_suite=""
last_compiling=""
results=()

while IFS= read -r line; do
    # Strip ANSI escape sequences (`forc` colorizes its output even when piped).
    while [[ $line =~ $ansi_re ]]; do
        line="${line/"${BASH_REMATCH[0]}"/}"
    done

    if [[ $line =~ ^tested\ --\ ([^[:space:]]+) ]]; then
        tested_suite="${BASH_REMATCH[1]}"
    fi

    if [[ $line =~ ^[[:space:]]*Compiling[[:space:]]+[A-Za-z]+[[:space:]]+([^[:space:]]+) ]]; then
        last_compiling="${BASH_REMATCH[1]}"
    fi

    if [[ $line =~ Finished[[:space:]]+release[[:space:]].*target\(s\)[[:space:]]+\[([0-9]+(\.[0-9]+)?)[[:space:]]*([A-Za-z]+)?\] ]]; then
        num="${BASH_REMATCH[1]}"
        unit="${BASH_REMATCH[3]}"

        # Prefer the `tested --` project (forc test), fall back to the last
        # `Compiling` project (forc build).
        suite="$tested_suite"
        [[ -z "$suite" ]] && suite="$last_compiling"

        # Convert the decimal (base-1000) unit back to an exact byte count.
        case "$unit" in
            "" | B) mult=1 ;;
            KB) mult=1000 ;;
            MB) mult=1000000 ;;
            GB) mult=1000000000 ;;
            TB) mult=1000000000000 ;;
            *) mult=1 ;;
        esac

        size="$(LC_ALL=C awk -v n="$num" -v m="$mult" 'BEGIN { printf "%.0f", n * m }')"

        # Only emit a row once we know which project the size belongs to.
        if [ -n "$suite" ]; then
            if [ "$1" = "" ]; then
                results+=("${suite},${size}")
            else
                results+=("${suite},${size},$1")
            fi
        fi

        # Reset so the next `Finished` line is attributed to its own project.
        tested_suite=""
        last_compiling=""
    fi
done

printf '%s\n' "${results[@]}" | sort
