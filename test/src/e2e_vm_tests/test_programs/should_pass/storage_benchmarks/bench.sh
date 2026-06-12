#!/usr/bin/env bash
#
# Runs all storage benchmarks: storage field benchmarks first
# (bench_storage_fields.sh), then StorageVec benchmarks
# (bench_storage_vec.sh).
#
# The output is printed to the console and also saved to a single file
# in the `perf_out` folder, named:
#
#   MMDDHHMMSS-<branch name>.all.txt
#
# where <branch name> is the normalized name of the current git branch.
#
# An optional filter limits the run to the benchmark projects whose
# names contain the filter as a substring.
#
# Usage:
#   ./bench.sh [<filter>]
#
# Examples:
#   ./bench.sh                # run all benchmark projects
#   ./bench.sh storage_vec    # run only the StorageVec projects
#   ./bench.sh s8             # run only storage_vec_s8 and storage_vec_s88
#
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

FILTER="${1:-}"

# ── Output file ─────────────────────────────────────────────────────

OUT_DIR="$SCRIPT_DIR/perf_out"
mkdir -p "$OUT_DIR"

TIMESTAMP=$(date +%m%d%H%M%S)
BRANCH=$(git -C "$SCRIPT_DIR" rev-parse --abbrev-ref HEAD | sed 's![^A-Za-z0-9._-]!_!g')
OUT_FILE="$OUT_DIR/${TIMESTAMP}-${BRANCH}.all.txt"

# ── Main ────────────────────────────────────────────────────────────

# BENCH_NO_SAVE tells the individual scripts not to save their own
# perf_out files; the combined output is saved here instead.
# BENCH_FILTER limits the run to the matching benchmark projects.
{
    BENCH_NO_SAVE=1 BENCH_FILTER="$FILTER" "$SCRIPT_DIR/bench_storage_fields.sh"
    BENCH_NO_SAVE=1 BENCH_FILTER="$FILTER" "$SCRIPT_DIR/bench_storage_vec.sh"
} | tee "$OUT_FILE"

echo "Results saved to: $OUT_FILE"
