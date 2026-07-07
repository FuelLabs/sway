#! /bin/bash

# All `std` tests must live in the in-language tests
# (`test/src/in_language_tests`) and not in `sway-lib-std` itself.
#
# This script searches `sway-lib-std` for any in-language unit tests
# (functions annotated with `#[test]` or `#[test(should_revert)]`) and
# prints every test it finds. It exits with a non-zero status if any
# test is found, so it can be used in CI to forbid tests in `sway-lib-std`.

set -euo pipefail

# Run from the directory containing this script, so the script can be
# invoked from anywhere (e.g. from the repository root in CI).
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

# Match the `#[test]` / `#[test(should_revert)]` attribute.
#
# We assume that `#[` and `test` always appear on the same line, and treat
# that as enough to denote a test. Whitespace is allowed anywhere within the
# attribute (e.g. `#  [   test ]`). The leading `^[[:space:]]*` anchor also
# ensures the attribute is not behind a `//` comment, since a comment would
# put `//` before the `#`.
#
# We use the POSIX `[[:space:]]` class rather than the `\s` shorthand: `\s`
# is a GNU extension that BSD/macOS `grep` does not support, so a pattern
# using `\s` would silently fail to match (and forbidden tests would slip
# through) when running this script locally on macOS.
#
# `grep` exits with 1 when there are no matches. We invert that here:
# no matches means no forbidden tests, which is the success case.
FOUND="$(grep -rn --include='*.sw' -E '^[[:space:]]*#[[:space:]]*\[[[:space:]]*test' src || true)"

if [[ -n "$FOUND" ]]; then
    count="$(printf '%s\n' "$FOUND" | wc -l)"
    count="$((count))"
    if [[ "$count" -eq 1 ]]; then
        noun="test"
    else
        noun="tests"
    fi
    echo "Found $count $noun in 'sway-lib-std'. All 'std' tests must be placed in the in-language tests ('test/src/in_language_tests'):"
    echo
    # For each matched attribute, print it together with the line right below
    # it (the test function declaration), both as `file:line: text`.
    while IFS= read -r match; do
        file="${match%%:*}"
        rest="${match#*:}"
        line="${rest%%:*}"
        text="${rest#*:}"
        next_line=$((line + 1))
        next_text="$(sed -n "${next_line}p" "$file")"
        printf '%s:%s: %s\n' "$file" "$line" "$text"
        printf '%s:%s: %s\n' "$file" "$next_line" "$next_text"
        echo
    done <<< "$FOUND"
    exit 1
fi

echo "No tests found in 'sway-lib-std'."
