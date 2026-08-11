#!/usr/bin/env bash

# Orchestrate the local test suite for the Sway compiler.
#
# Usage: ./scripts/test/run-tests.sh [--lsp]
#
#   --lsp   Additionally run the (slow) `sway-lsp` tests.
#
# The script runs, in order:
#   1. All tests:                cargo r -r -p test
#   2. E2E tests (release):      cargo r -r -p test -- --release -ke2e
#                                (fuel-core is started/stopped automatically)
#   3. SDK harness tests:        build the contracts, then run the tests
#   4. In-language tests:        debug and release
#   5. sway-ir unit tests:       cargo test -p sway-ir
#   6. sway-lsp tests:           cargo test -p sway-lsp   (only with --lsp)
#
# It fails fast: the first failing step aborts the run and the remaining steps
# are skipped. A per-step and total timing breakdown is printed at the end, and
# an OS notification (Linux: notify-send, macOS: terminal-notifier/osascript) is
# sent when the run finishes or fails.
#
# Works on Linux and macOS (bash 3.2+).

set -uo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
cd "$REPO_ROOT"

# ---------------------------------------------------------------------------
# Argument parsing
# ---------------------------------------------------------------------------
RUN_LSP=false
for arg in "$@"; do
    case "$arg" in
        --lsp)
            RUN_LSP=true
            ;;
        -h|--help)
            # Print the leading comment block (skipping the shebang line).
            awk 'NR==1{next} /^#/{started=1; sub(/^# ?/,""); print; next} started{exit}' "${BASH_SOURCE[0]}"
            exit 0
            ;;
        *)
            echo "Error: unknown argument: $arg" >&2
            echo "Usage: $0 [--lsp]" >&2
            exit 2
            ;;
    esac
done

# ---------------------------------------------------------------------------
# Timing / bookkeeping
# ---------------------------------------------------------------------------
OVERALL_START=$SECONDS
FINISHED_OK=false
CURRENT_STEP=""
FUEL_CORE_PID=""

STEP_NAMES=()
STEP_DURS=()
STEP_STATUS=()

fmt_dur() {
    local s=$1
    printf '%dh %02dm %02ds' $(( s / 3600 )) $(( (s % 3600) / 60 )) $(( s % 60 ))
}

# ---------------------------------------------------------------------------
# OS notification (best-effort; never fails the run)
# ---------------------------------------------------------------------------
notify() {
    local title="$1"
    local message="$2"
    if [[ "$(uname)" == "Darwin" ]]; then
        if command -v terminal-notifier >/dev/null 2>&1; then
            terminal-notifier -title "$title" -message "$message" >/dev/null 2>&1 || true
        else
            osascript -e "display notification \"${message//\"/\\\"}\" with title \"${title//\"/\\\"}\"" >/dev/null 2>&1 || true
        fi
    else
        if command -v notify-send >/dev/null 2>&1; then
            notify-send "$title" "$message" >/dev/null 2>&1 || true
        fi
    fi
}

# ---------------------------------------------------------------------------
# fuel-core lifecycle (used by the E2E step)
# ---------------------------------------------------------------------------
stop_fuel_core() {
    if [[ -n "$FUEL_CORE_PID" ]] && kill -0 "$FUEL_CORE_PID" 2>/dev/null; then
        echo "Stopping fuel-core (pid $FUEL_CORE_PID)..."
        kill "$FUEL_CORE_PID" 2>/dev/null || true
        wait "$FUEL_CORE_PID" 2>/dev/null || true
    fi
    FUEL_CORE_PID=""
}

wait_for_fuel_core() {
    # fuel-core's GraphQL service listens on 127.0.0.1:4000 by default.
    local port=4000
    local tries=120 # up to ~60s
    local i
    for (( i = 0; i < tries; i++ )); do
        if ! kill -0 "$FUEL_CORE_PID" 2>/dev/null; then
            echo "Error: fuel-core exited before becoming ready." >&2
            return 1
        fi
        if (exec 3<>"/dev/tcp/127.0.0.1/$port") 2>/dev/null; then
            exec 3>&- 3<&- 2>/dev/null || true
            return 0
        fi
        sleep 0.5
    done
    echo "Error: timed out waiting for fuel-core to listen on port $port." >&2
    return 1
}

# ---------------------------------------------------------------------------
# Reporting on exit (breakdown + notification)
# ---------------------------------------------------------------------------
print_breakdown() {
    local total=$(( SECONDS - OVERALL_START ))
    echo ""
    echo "=================================================================="
    echo "Test run breakdown"
    echo "=================================================================="
    if [[ ${#STEP_NAMES[@]} -gt 0 ]]; then
        local i
        for i in "${!STEP_NAMES[@]}"; do
            printf '  %-30s %-6s %s\n' "${STEP_NAMES[$i]}" "${STEP_STATUS[$i]}" "$(fmt_dur "${STEP_DURS[$i]}")"
        done
    else
        echo "  (no steps were run)"
    fi
    echo "------------------------------------------------------------------"
    printf '  %-30s %-6s %s\n' "TOTAL" "" "$(fmt_dur "$total")"
    echo "=================================================================="
}

finish() {
    local rc=$?
    stop_fuel_core
    print_breakdown
    if [[ "$FINISHED_OK" == true ]]; then
        notify "Sway tests: PASSED" "All test steps completed in $(fmt_dur $(( SECONDS - OVERALL_START )))."
    else
        notify "Sway tests: FAILED" "Failed at step: ${CURRENT_STEP:-unknown}."
    fi
}
trap finish EXIT
trap 'exit 130' INT
trap 'exit 143' TERM

# ---------------------------------------------------------------------------
# Step runner
# ---------------------------------------------------------------------------
# Usage: run_step "Human readable name" command args...
# The command can be an external program or a shell function.
run_step() {
    local name="$1"
    shift
    CURRENT_STEP="$name"

    echo ""
    echo "=================================================================="
    echo ">> $name"
    echo "=================================================================="

    local start=$SECONDS
    "$@"
    local rc=$?
    local dur=$(( SECONDS - start ))

    STEP_NAMES+=("$name")
    STEP_DURS+=("$dur")

    if [[ $rc -ne 0 ]]; then
        STEP_STATUS+=("FAIL")
        echo ""
        echo "!! Step failed: $name (exit code $rc). Aborting." >&2
        exit "$rc"
    fi

    STEP_STATUS+=("PASS")
    echo "-- Done: $name ($(fmt_dur "$dur"))"
}

# ---------------------------------------------------------------------------
# Composite steps
# ---------------------------------------------------------------------------
step_e2e_release() {
    if ! command -v fuel-core >/dev/null 2>&1; then
        echo "Error: \`fuel-core\` not found on PATH. Install it to run the E2E tests." >&2
        return 1
    fi

    local fuel_core_log
    fuel_core_log="$(mktemp)"

    echo "Starting fuel-core (logs: $fuel_core_log)..."
    fuel-core run --debug --db-type=in-memory >"$fuel_core_log" 2>&1 &
    FUEL_CORE_PID=$!

    if ! wait_for_fuel_core; then
        echo "---- fuel-core log ----" >&2
        cat "$fuel_core_log" >&2 || true
        stop_fuel_core
        rm -f "$fuel_core_log"
        return 1
    fi
    echo "fuel-core is ready (pid $FUEL_CORE_PID)."

    cargo r -r -p test -- --release -ke2e
    local rc=$?

    stop_fuel_core
    rm -f "$fuel_core_log"
    return $rc
}

step_sdk_harness() {
    cargo run --locked --release -p forc -- build --locked \
        --path ./test/src/sdk-harness \
        --output-directory ./test/src/sdk-harness/out \
        && cargo test --locked --release \
            --manifest-path ./test/src/sdk-harness/Cargo.toml \
            -- --skip can_get_predicate_address --nocapture
}

# ---------------------------------------------------------------------------
# Run everything
# ---------------------------------------------------------------------------
run_step "All tests"                 cargo r -r -p test
run_step "E2E tests (release)"       step_e2e_release
run_step "SDK harness tests"         step_sdk_harness
run_step "In-language tests"         ./test/src/in_language_tests/run_in_language_tests.sh
run_step "In-language tests (release)" ./test/src/in_language_tests/run_in_language_tests.sh --release
run_step "sway-ir unit tests"        cargo test -p sway-ir

if [[ "$RUN_LSP" == true ]]; then
    run_step "sway-lsp tests"        cargo test -p sway-lsp
fi

FINISHED_OK=true
echo ""
echo "All test steps passed."
exit 0
