#!/usr/bin/env bash

# Run `forc test` individually for each project under test_programs/.
#
# Usage: ./run_in_language_tests.sh [--sequential] [--print-output] [--filter REGEX] [extra `forc test` args, e.g. --release --experimental ...]
#
# Note: --error-on-warnings arg is always passed to `forc test`.
#       When --filter is provided, projects are selected if either:
#       - project directory name matches REGEX, or
#       - any *.sw file in the project matches REGEX.
#
# Run modes:
#   Default (parallel): projects are run concurrently and only a concise per-project
#       result is printed (green check mark on success, red cross mark on failure).
#       The full output of failing projects is printed at the end. This is the
#       default because it is fast, which suits CI and local "do all tests pass?"
#       checks. Concurrency defaults to half of the available CPU cores (at least 1)
#       and can be overridden via the PARALLEL_JOBS environment variable.
#       (For backward compatibility, an explicit `--parallel` is still accepted.)
#   --sequential: projects are run one after another and the full `forc test` output
#       (compilation and tests output) is printed as it runs.
#
# --print-output: print the full `forc test` output of every project at the end,
#       in a stable (sorted) order. This makes the otherwise concise --parallel
#       mode usable for gas-usage extraction: run with `--parallel --print-output`
#       and pipe the output into `scripts/perf/extract-gas-usages.sh`. In the
#       default sequential mode the full output is already printed, so this flag
#       has no additional effect there.
#
# The script continues even when individual test projects fail, and exits with
# a non-zero code at the end if any project failed.

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
TEST_PROGRAMS_DIR="$SCRIPT_DIR/test_programs"

FILTER_REGEX=""
PARALLEL=true
PRINT_OUTPUT=false
FORC_TEST_ARGS=()

while [[ $# -gt 0 ]]; do
    case "$1" in
        --sequential)
            PARALLEL=false
            shift
            ;;
        --parallel)
            # Parallel is now the default; accepted for backward compatibility.
            PARALLEL=true
            shift
            ;;
        --print-output)
            PRINT_OUTPUT=true
            shift
            ;;
        --filter)
            if [[ $# -lt 2 ]]; then
                echo "Error: --filter requires a regex argument" >&2
                exit 2
            fi
            FILTER_REGEX="$2"
            shift 2
            ;;
        --filter=*)
            FILTER_REGEX="${1#--filter=}"
            shift
            ;;
        *)
            FORC_TEST_ARGS+=("$1")
            shift
            ;;
    esac
done

if [[ -n "$FILTER_REGEX" ]]; then
    # Validate regex syntax once up front for clearer failures.
    printf '' | grep -Eq -- "$FILTER_REGEX" 2>/dev/null
    regex_status=$?
    if [[ $regex_status -eq 2 ]]; then
        echo "Error: invalid regex provided via --filter: $FILTER_REGEX" >&2
        exit 2
    fi
fi

# Always use the `forc` binary built in the repository root (3 levels above here:
# test/src/in_language_tests/ -> sway/).
REPO_ROOT="$(cd "$SCRIPT_DIR/../../.." && pwd)"
FORC="$REPO_ROOT/target/release/forc"
if [[ ! -x "$FORC" ]]; then
    echo "Error: \`forc\` binary not found at $FORC" >&2
    echo "Build it with: cargo build --release -p forc" >&2
    exit 1
fi

# Project directory names under test_programs/ to skip.
# Currently empty, but left for possible future use.
EXCLUDED_PROJECTS=(
)

failed=()
passed=()

start_time=$SECONDS

# The package name (suite) as printed by `forc test` when run on a workspace.
# When `forc test` is run on a single package (as this script does, via `--path`),
# it does NOT print the `tested -- <suite>` line that the gas-usage extraction relies
# on to prefix test names with their suite. We therefore emit that line ourselves,
# using the package `name` from `Forc.toml` (falling back to the directory name).
suite_name() {
    local project_dir="$1"
    local project_name="$2"
    local name
    name="$(grep -E '^[[:space:]]*name[[:space:]]*=' "$project_dir/Forc.toml" 2>/dev/null \
        | head -n1 \
        | sed -E 's/^[[:space:]]*name[[:space:]]*=[[:space:]]*"?([^"]*)"?[[:space:]]*$/\1/')"
    if [[ -n "$name" ]]; then
        printf '%s' "$name"
    else
        printf '%s' "$project_name"
    fi
}

should_run_project() {
    local project_name="$1"
    local project_dir="$2"

    if [[ -z "$FILTER_REGEX" ]]; then
        return 0
    fi

    if [[ "$project_name" =~ $FILTER_REGEX ]]; then
        return 0
    fi

    while IFS= read -r -d '' sw_file; do
        if grep -Eq -- "$FILTER_REGEX" "$sw_file"; then
            return 0
        fi
    done < <(find "$project_dir" -type f -name "*.sw" -print0)

    return 1
}

# Collect the projects to run (in a stable, sorted order) before running them,
# so that both the sequential and parallel modes operate on the same list.
project_names=()
project_dirs=()
while IFS= read -r -d '' forc_toml; do
    project_dir="$(dirname "$forc_toml")"
    project_name="$(basename "$project_dir")"

    if [[ " ${EXCLUDED_PROJECTS[*]} " == *" ${project_name} "* ]]; then
        continue
    fi

    if ! should_run_project "$project_name" "$project_dir"; then
        continue
    fi

    project_names+=("$project_name")
    project_dirs+=("$project_dir")
# Projects may be nested in grouping folders (e.g. `storage/`, `storage/storage_vec/`),
# so discover `Forc.toml` at any depth, skipping build output (`out/`) directories.
done < <(find "$TEST_PROGRAMS_DIR" -mindepth 2 -name "Forc.toml" -not -path "*/out/*" -print0 | sort -z)

if [[ "$PARALLEL" == true ]]; then
    # Parallel mode: run all projects concurrently, capturing each project's
    # output to its own log file. Only a concise per-project result is printed
    # live (green check on success, red cross on failure). Full output of any
    # failing project is printed at the end.
    GREEN='\033[0;32m'
    RED='\033[0;31m'
    NC='\033[0m'

    # Default concurrency to half of the available cores (at least 1) to leave
    # headroom on the machine. Can be overridden via the PARALLEL_JOBS env var.
    # `nproc` is Linux; `sysctl -n hw.ncpu` is the macOS equivalent.
    cores="$(nproc 2>/dev/null || sysctl -n hw.ncpu 2>/dev/null || echo 4)"
    PARALLEL_JOBS="${PARALLEL_JOBS:-$(( cores / 2 ))}"
    (( PARALLEL_JOBS < 1 )) && PARALLEL_JOBS=1

    # `wait -n` (wait for any single job to finish) requires Bash >= 4.3.
    # macOS ships Bash 3.2, so fall back to polling there.
    if (( BASH_VERSINFO[0] > 4 || (BASH_VERSINFO[0] == 4 && BASH_VERSINFO[1] >= 3) )); then
        HAVE_WAIT_N=true
    else
        HAVE_WAIT_N=false
    fi

    wait_for_slot() {
        if [[ "$HAVE_WAIT_N" == true ]]; then
            wait -n 2>/dev/null || true
        else
            sleep 0.2
        fi
    }

    logs_dir="$(mktemp -d)"
    trap 'rm -rf "$logs_dir"' EXIT

    run_one() {
        local project_name="$1"
        local project_dir="$2"
        local log_file="$logs_dir/$project_name.log"

        # Emit the `tested -- <suite>` line into the log so it is self-contained and
        # gas-usage extraction can prefix test names with their suite when the logs
        # are later printed (see `suite_name` above for why this is needed).
        {
            echo ""
            echo "tested -- $(suite_name "$project_dir" "$project_name")"
            echo ""
        } > "$log_file"

        if "$FORC" test --error-on-warnings "${FORC_TEST_ARGS[@]}" --path "$project_dir" >> "$log_file" 2>&1; then
            printf '%b %s\n' "${GREEN}✓${NC}" "$project_name"
            echo "pass" > "$logs_dir/$project_name.status"
        else
            printf '%b %s\n' "${RED}✗${NC}" "$project_name"
            echo "fail" > "$logs_dir/$project_name.status"
        fi
    }

    echo "Running ${#project_names[@]} test projects in parallel (up to $PARALLEL_JOBS at a time)..."
    echo ""

    for i in "${!project_names[@]}"; do
        # Throttle: wait for a free slot before launching the next job.
        while (( $(jobs -rp | wc -l) >= PARALLEL_JOBS )); do
            wait_for_slot
        done
        run_one "${project_names[$i]}" "${project_dirs[$i]}" &
    done

    # Wait for all remaining jobs to finish.
    wait

    # Build pass/fail lists in the stable project order.
    for project_name in "${project_names[@]}"; do
        if [[ "$(cat "$logs_dir/$project_name.status" 2>/dev/null)" == "pass" ]]; then
            passed+=("$project_name")
        else
            failed+=("$project_name")
        fi
    done

    if [[ "$PRINT_OUTPUT" == true ]]; then
        # Print the full output of every project (in stable order) so that the
        # otherwise concise parallel mode can be piped into gas-usage extraction.
        # Each log already starts with its `tested -- <suite>` line.
        echo ""
        echo "================================================"
        echo "Full output of all test projects:"
        echo "================================================"
        for project_name in "${project_names[@]}"; do
            echo ""
            cat "$logs_dir/$project_name.log" 2>/dev/null
        done
    elif [[ ${#failed[@]} -gt 0 ]]; then
        # Print the full output of any failing project to aid debugging on CI.
        echo ""
        echo "================================================"
        echo "Output of failing test projects:"
        echo "================================================"
        for project_name in "${failed[@]}"; do
            echo ""
            echo "==> $project_name"
            cat "$logs_dir/$project_name.log" 2>/dev/null
        done
    fi
else
    # Sequential mode (opt-in via --sequential): run projects one by one and
    # print the full `forc test` output as it runs.
    for i in "${!project_names[@]}"; do
        project_name="${project_names[$i]}"
        project_dir="${project_dirs[$i]}"

        echo ""
        echo "==> Testing: $project_name"

        # Emit the `tested -- <suite>` line so that gas-usage extraction can prefix
        # test names with their suite (see `suite_name` above for why this is needed).
        echo ""
        echo "tested -- $(suite_name "$project_dir" "$project_name")"
        echo ""

        if "$FORC" test --error-on-warnings "${FORC_TEST_ARGS[@]}" --path "$project_dir"; then
            passed+=("$project_name")
        else
            echo "FAILED: $project_name" >&2
            failed+=("$project_name")
        fi
    done
fi

elapsed=$(( SECONDS - start_time ))
elapsed_str="$(printf '%dh %02dm %02ds' $(( elapsed / 3600 )) $(( (elapsed % 3600) / 60 )) $(( elapsed % 60 )))"

echo ""
echo "================================================"
echo "Results: ${#passed[@]} passed, ${#failed[@]} failed"
echo "Total run time: ${elapsed_str}"
echo "================================================"

if [[ ${#failed[@]} -gt 0 ]]; then
    echo ""
    echo "Failed tests (${#failed[@]}):"
    for f in "${failed[@]}"; do
        echo "  - $f"
    done
    exit 1
fi

echo "All ${#passed[@]} tests passed!"
