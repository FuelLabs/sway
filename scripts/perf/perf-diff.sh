#!/bin/bash

# This script compares performance data (gas usages and bytecode sizes) from two CSV files.
# CSV files must have two columns, the test name and the performance data, and the test
# names must be the same and in the same order in both files.
# The result of the comparison can be printed either as a Markdown table or a CSV file.
# Usage: `perf-diff.sh <before>.csv <after>.csv [md|csv]`.

if [ "$#" -lt 2 ] || [ "$#" -gt 3 ]; then
    echo "Usage: $0 <before>.csv <after>.csv [md|csv]"
    exit 1
fi

before_file="$1"
after_file="$2"
output_format="${3:-md}"

if [[ "$output_format" != "md" && "$output_format" != "csv" ]]; then
    echo "ERROR: Invalid output format '$output_format'. Output format must be either 'md' or 'csv'."
    exit 2
fi

# A data row is `<name>,<integer>`. The value must be an integer; the name may
# contain any characters except a comma.
# Lines like `test,gas` (the header) have non-integer and are ignored.
awk -v fmt="$output_format" '
BEGIN { FS = "," }
FNR == NR {
    # ── before file ──
    if ($0 ~ /^[[:space:]]*Running:[[:space:]]+/) {
        bsec = $0
        sub(/^[[:space:]]*Running:[[:space:]]+/, "", bsec)
        sub(/[[:space:]]+$/, "", bsec)
        if (!(bsec in bsec_seen)) { bsec_seen[bsec] = 1; bsec_order[++bnsec] = bsec }
        next
    }
    if ($0 ~ /^[^,]+,-?[0-9]+$/) {
        name = $1
        if (!(bsec in bsec_seen)) { bsec_seen[bsec] = 1; bsec_order[++bnsec] = bsec }
        key = bsec SUBSEP name
        if (!(key in bval)) { brow_count[bsec]++; brow_order[bsec, brow_count[bsec]] = name }
        bval[key] = $2
    }
    next
}
{
    # ── after file ──
    if ($0 ~ /^[[:space:]]*Running:[[:space:]]+/) {
        asec = $0
        sub(/^[[:space:]]*Running:[[:space:]]+/, "", asec)
        sub(/[[:space:]]+$/, "", asec)
        if (!(asec in asec_seen)) { asec_seen[asec] = 1; asec_order[++ansec] = asec }
        next
    }
    if ($0 ~ /^[^,]+,-?[0-9]+$/) {
        name = $1
        if (!(asec in asec_seen)) { asec_seen[asec] = 1; asec_order[++ansec] = asec }
        key = asec SUBSEP name
        if (!(key in aval)) { arow_count[asec]++; arow_order[asec, arow_count[asec]] = name }
        aval[key] = $2
    }
}
END {
    if (fmt == "csv") print "Test,Before,After,Percentage"
    else {
        print "| Test | Before | After | Percentage |"
        print "| ---- | -----: | ----: | ---------: |"
    }

    added = 0
    removed = 0

    # Sections present in the before file, in before order.
    for (i = 1; i <= bnsec; i++) {
        sec = bsec_order[i]
        nb = brow_count[sec]
        for (j = 1; j <= nb; j++) {
            name = brow_order[sec, j]
            bkey = sec SUBSEP name
            akey = sec SUBSEP name
            if (akey in aval) {
                if (bval[bkey] != aval[akey]) emit(sec, name, bval[bkey], aval[akey], 1)
            } else {
                emit(sec, name, bval[bkey], "", 0)
                removed++
            }
        }
        # Tests that exist only in the after file, in after order.
        na = arow_count[sec]
        for (j = 1; j <= na; j++) {
            an = arow_order[sec, j]
            if (!((sec SUBSEP an) in bval)) {
                emit(sec, an, "", aval[sec SUBSEP an], 0)
                added++
            }
        }
    }

    # Sections that exist only in the after file, in after order.
    for (i = 1; i <= ansec; i++) {
        sec = asec_order[i]
        if (sec in bsec_seen) continue
        na = arow_count[sec]
        for (j = 1; j <= na; j++) {
            an = arow_order[sec, j]
            emit(sec, an, "", aval[sec SUBSEP an], 0)
            added++
        }
    }

    if (added > 0 || removed > 0) {
        printf("Note: %d test(s) added, %d test(s) removed; excluded from statistics.\n", added, removed) > "/dev/stderr"
    }
}
function emit(sec, name, bv, av, matched,    disp, diff, pct) {
    disp = (sec == "" ? name : sec "/" name)
    if (matched) {
        diff = bv - av
        if (bv == 0) pct = "NaN"
        else pct = sprintf("%.2f", (diff / bv) * 100)
        if (fmt == "csv") printf("%s,%s,%s,%s\n", disp, bv, av, pct)
        else printf("| %s | %s | %s | %s%% |\n", disp, bv, av, pct)
    } else {
        if (fmt == "csv") printf("%s,%s,%s,\n", disp, bv, av)
        else printf("| %s | %s | %s |  |\n", disp, bv, av)
    }
}
' "$before_file" "$after_file"
