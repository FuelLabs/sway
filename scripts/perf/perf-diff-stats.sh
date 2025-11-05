#!/bin/bash

# This script calculates a simple statistical summary of performance differences
# collected from two CSV files, by using `perf-diff.sh` to compute the diffs first.
#
# The summary includes count, average, median, max, min of improvements and regressions
# (percentage changes), and outputs the result as a Markdown table.
#
# Usage, e.g.: `cat <output of perf-diff.sh> | perf-diff-stats.sh`.

set -euo pipefail

# Force C locale so both parsing and sorting use '.' as decimal separator.
export LC_ALL=C

tmp_all="$(mktemp)"; trap 'rm -f "$tmp_all" "$posf" "$negf" "$pos_sorted" "$neg_sorted"' EXIT
posf="$(mktemp)"
negf="$(mktemp)"
pos_sorted="$(mktemp)"
neg_sorted="$(mktemp)"

# Stash stdin for multiple passes.
cat > "$tmp_all"

# Extract normalized numeric values from the fourth column.
awk -F',' -v POS="$posf" -v NEG="$negf" '
NR==1 { next }  # skip header
{
  v = $4
  v += 0
  if (v > 0) {
    print v >> POS
  } else if (v < 0) {
    print (-v) >> NEG
  }
}
' "$tmp_all"

# Calculate stats (avg, min, max, count) for a one-column file.
stats() {
  local file="$1"
  if [[ ! -s "$file" ]]; then
    # no values
    echo "— — — 0"
    return
  fi
  awk '
    NR==1 { min=$1; max=$1; sum=$1; c=1; next }
           { if($1<min) min=$1; if($1>max) max=$1; sum+=$1; c++ }
    END    { printf("%.2f %.2f %.2f %d\n", sum/c, min, max, c) }
  ' "$file"
}

# Calculate median for a one-column file (sorted numerically first).
median() {
  local file="$1"
  if [[ ! -s "$file" ]]; then
    echo "—"
    return
  fi
  sort -n "$file" > "$file.sorted.$$"
  awk '
    { a[++n]=$1 }
    END {
      if (n==0) { print "—"; exit }
      if (n%2==1) {
        printf("%.2f\n", a[(n+1)/2])
      } else {
        printf("%.2f\n", (a[n/2]+a[n/2+1])/2)
      }
    }
  ' "$file.sorted.$$"
  rm -f "$file.sorted.$$"
}

# Compute stats.
read -r p_avg p_min p_max p_cnt <<<"$(stats "$posf")"
read -r n_avg n_min n_max n_cnt <<<"$(stats "$negf")"

# Compute medians.
cp "$posf" "$pos_sorted" 2>/dev/null || true
cp "$negf" "$neg_sorted" 2>/dev/null || true
p_med="$(median "$pos_sorted")"
n_med="$(median "$neg_sorted")"

# Print a table cell value or em-dash.
fmt_cnt() { [[ "$1" == "0" ]] && echo "—" || printf "%d" "$1"; }
fmt_imp() { [[ "$1" == "—" ]] && echo "—" || printf "%.2f%%" "$1"; }
fmt_reg() { [[ "$1" == "—" ]] && echo "—" || printf "%.2f%%" "-$1"; }

# Print the summary table.
echo "|   | Improvements | Regressions |"
echo "| - | -: | -: |"
printf "| Count   | %s | %s |\n" "$(fmt_cnt "$p_cnt")" "$(fmt_cnt "$n_cnt")"
printf "| Average | %s | %s |\n" "$(fmt_imp "$p_avg")" "$(fmt_reg "$n_avg")"
printf "| Median  | %s | %s |\n" "$(fmt_imp "$p_med")" "$(fmt_reg "$n_med")"
printf "| Max     | %s | %s |\n" "$(fmt_imp "$p_max")" "$(fmt_reg "$n_max")"
printf "| Min     | %s | %s |\n" "$(fmt_imp "$p_min")" "$(fmt_reg "$n_min")"
