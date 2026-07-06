#!/usr/bin/env bash
#
# Computes per-project statistics from a perf-diff CSV file.
#
# Usage:
#   ./perf_diff_stats.sh <name>.perf-diff.csv
#
# Output:
#   <name>.perf-diff-stats.md
#
# For each project, produces a table with count, average, median,
# max, and min of improvement and regression percentages.
#
set -euo pipefail

# ── Argument validation ─────────────────────────────────────────────

if [[ $# -ne 1 ]]; then
    echo "Usage: $0 <name>.perf-diff.csv" >&2
    exit 1
fi

CSV_FILE="$1"

if [[ ! -f "$CSV_FILE" ]]; then
    echo "ERROR: file not found: $CSV_FILE" >&2
    exit 1
fi

if [[ "$CSV_FILE" != *.perf-diff.csv ]]; then
    echo "ERROR: expected a .perf-diff.csv file, got: $CSV_FILE" >&2
    exit 1
fi

# Derive output filename: replace .perf-diff.csv with .perf-diff-stats.md
MD_FILE="${CSV_FILE%.perf-diff.csv}.perf-diff-stats.md"

# ── Parse CSV and compute stats ─────────────────────────────────────

# Use awk to group by project, separate improvements (positive %)
# from regressions (negative %), compute count, average, median,
# max, min for each group, and emit markdown tables.

LC_NUMERIC=C awk -F, '
BEGIN {
    n_projects = 0
}

NR == 1 { next }

{
    project = $1
    pct_str = $6
    if (pct_str == "N/A") next
    gsub(/%$/, "", pct_str)
    pct = pct_str + 0.0

    if (!(project in seen_project)) {
        seen_project[project] = 1
        project_order[n_projects++] = project
        imp_n[project] = 0
        reg_n[project] = 0
    }

    if (pct > 0) {
        imp[project, imp_n[project]++] = pct
    } else if (pct < 0) {
        reg[project, reg_n[project]++] = pct
    }
}

function sort_arr(arr, proj, n,    i, j, tmp) {
    for (i = 1; i < n; i++) {
        tmp = arr[proj, i]
        j = i - 1
        while (j >= 0 && arr[proj, j] > tmp) {
            arr[proj, j + 1] = arr[proj, j]
            j--
        }
        arr[proj, j + 1] = tmp
    }
}

function fmt_pct(v) { return sprintf("%.2f%%", v) }

function calc_median(arr, proj, n) {
    if (n == 0) return "\xe2\x80\x94"
    sort_arr(arr, proj, n)
    if (n % 2 == 1) return fmt_pct(arr[proj, int(n/2)])
    return fmt_pct((arr[proj, int(n/2)-1] + arr[proj, int(n/2)]) / 2.0)
}

function calc_avg(arr, proj, n,    i, s) {
    if (n == 0) return "\xe2\x80\x94"
    s = 0; for (i = 0; i < n; i++) s += arr[proj, i]
    return fmt_pct(s / n)
}

function calc_max(arr, proj, n,    i, m) {
    if (n == 0) return "\xe2\x80\x94"
    m = arr[proj, 0]; for (i = 1; i < n; i++) if (arr[proj, i] > m) m = arr[proj, i]
    return fmt_pct(m)
}

function calc_min(arr, proj, n,    i, m) {
    if (n == 0) return "\xe2\x80\x94"
    m = arr[proj, 0]; for (i = 1; i < n; i++) if (arr[proj, i] < m) m = arr[proj, i]
    return fmt_pct(m)
}

function cnt(n) { return (n == 0) ? "\xe2\x80\x94" : n }

function max2(a, b) { return (a > b) ? a : b }

END {
    for (p = 0; p < n_projects; p++) {
        proj = project_order[p]
        in_ = imp_n[proj]; rn = reg_n[proj]

        v[0] = cnt(in_);               w[0] = cnt(rn)
        v[1] = calc_avg(imp, proj, in_);  w[1] = calc_avg(reg, proj, rn)
        v[2] = calc_median(imp, proj, in_); w[2] = calc_median(reg, proj, rn)
        v[3] = calc_max(imp, proj, in_);  w[3] = calc_min(reg, proj, rn)
        v[4] = calc_min(imp, proj, in_);  w[4] = calc_max(reg, proj, rn)

        labels[0] = "Count"; labels[1] = "Average"; labels[2] = "Median"
        labels[3] = "Max"; labels[4] = "Min"

        lw = 7  # length("Average")
        iw = 12 # length("Improvements")
        rw = 11 # length("Regressions")
        for (i = 0; i < 5; i++) {
            lw = max2(lw, length(labels[i]))
            iw = max2(iw, length(v[i]))
            rw = max2(rw, length(w[i]))
        }

        printf "## %s\n\n", proj
        printf "| %-*s | %*s | %*s |\n", lw, "", iw, "Improvements", rw, "Regressions"

        # Separator: left-align col 1, right-align cols 2 & 3.
        printf "| "
        for (i = 0; i < lw; i++) printf "-"
        printf " | "
        for (i = 0; i < iw - 1; i++) printf "-"
        printf ": | "
        for (i = 0; i < rw - 1; i++) printf "-"
        printf ": |\n"

        for (i = 0; i < 5; i++) {
            printf "| %-*s | %*s | %*s |\n", lw, labels[i], iw, v[i], rw, w[i]
        }
        printf "\n"
    }
}
' "$CSV_FILE" > "$MD_FILE"

echo "Generated: $MD_FILE"
