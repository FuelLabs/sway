# list all available recipes
default:
  just --list --unsorted

[group('ci')]
[confirm("Do you want to install cargo-sort, cargo-generate and cargo-udeps from crates.io?")]
install-ci-check:
    cargo install cargo-sort
    cargo install cargo-generate
    cargo install cargo-udeps

[group('ci')]
ci-check:
    bash ./ci_checks.sh

[group('automation')]
[confirm("Do you want to bump all fuel maintained dependencies?")]
update-fuel-dependencies:
    bash ./update_fuel_dependencies.sh

[group('automation')]
[confirm("Do you want to automatically update contractIds in this repo?")]
update-contract-ids:
    bash ./test/update-contract-ids.sh

[group('automation')]
bisect-forc path command:
    bash ./scripts/bisect-forc/bisect-forc.sh "{{path}}" "{{command}}"

# The `benchmark` group contains recipes related to benchmarking the Sway compiler, e.g., compilation times.

[group('benchmark')]
benchmark:
    bash ./benchmark.sh

[group('benchmark')]
benchmark-tests:
    bash ./test/bench.sh

# The `performance` group contains recipes related to benchmarking the performance of compiled code:
# gas usages and bytecode sizes.

alias pe2e := perf-e2e
# collect gas usages and bytecode sizes from E2E tests
[group('performance')]
perf-e2e filter='':
    cargo r -r -p test -- --release --kind e2e --perf-only --perf --gas-costs built-in {{filter}}

alias pil := perf-in-lang
# collect gas usages from in-language tests
[group('performance')]
perf-in-lang filter='':
    #!/usr/bin/env bash
    branch=$(git rev-parse --abbrev-ref HEAD 2>/dev/null); [[ "$branch" == "HEAD" || -z "$branch" ]] && branch="unknown-branch"; branch=${branch//\//-};
    outfile="./test/perf_out/$(date '+%m%d%H%M%S')-in-language-gas-usages-release-$branch.csv"
    cargo r -r -p forc -- test --release --path ./test/src/in_language_tests --gas-costs built-in {{filter}} | ./scripts/perf/extract-gas-usages.sh > "$outfile"
    echo "Gas usages written to:      $outfile"

alias pa := perf-all
# collect gas usages and bytecode sizes from all tests (E2E and in-language)
[group('performance')]
perf-all filter='': (perf-e2e filter) (perf-in-lang filter)

alias pd := perf-diff
# generate performance diff between two CSV files
[group('performance')]
perf-diff before after format='md':
    ./scripts/perf/perf-diff.sh "{{before}}" "{{after}}" "{{format}}"

alias pdl := perf-diff-latest
# generate performance diffs between the latest two CSV files per testing category
[group('performance')]
perf-diff-latest format='md':
    ./scripts/perf/perf-diff-latest.sh "{{format}}"

# This recipe should be used on snapshot tests that contain gas usages from `forc test`.
# It will extract gas usages from versions of the test's `stdout.snap` file
# that are within of the `revision_range`, and output a CSV pivot table
# or an interactive HTML report, depending on the `format`.
# path:           repo path to `stdout.snap` file to extract gas usage from, e.g.: `test/src/e2e_vm_tests/path_to/some_test/stdout.snap`
# revision_range: Git revision range to consider when collecting historic gas usages, e.g.: `HEAD~10..HEAD` (default: all revisions)
# format:         output format, either `csv` or `html` (default: `csv`)
# open:           for `html` output, `-o` opens the report in the default browser

alias psh := perf-snapshot-historical
# collect historic gas usages from a snapshot test that has a `forc test` output
[linux]
[group('performance')]
perf-snapshot-historical path revision_range='' format='csv' open='':
    #!/usr/bin/env bash
    if [[ "{{format}}" != "csv" && "{{format}}" != "html" ]]; then
        echo "ERROR: Invalid output format '{{format}}'.  Output format must be either 'csv' or 'html'."
        exit 1
    fi

    now_ts=$(date '+%m%d%H%M%S')
    outfile="./test/perf_out/$now_ts-snapshot-gas-usages-historical-$(basename "$(dirname "{{path}}")")"

    echo "test,gas,commit" > "$outfile.csv"

    for HASH in `git log --format='%H' {{revision_range}} -- {{path}}`; do
        TIMESTAMP=$(git show -s --format='%as-%ct-%H' "$HASH")
        git --no-pager show "$HASH:{{path}}" | bash -c "scripts/perf/extract-gas-usages.sh $TIMESTAMP" >> "$outfile.csv"
    done

    echo "Historical gas usages written to: $outfile.csv"

    if [ "{{format}}" = "html" ]; then
        ./scripts/csv2html/csv2html.sh "$outfile.csv" >> "$outfile.html"
        echo "Pivot table written to:           $outfile.html"
        if [ -n "{{open}}" ]; then
            if which xdg-open &>> /dev/null
            then
                xdg-open "$outfile.html"
            elif which gnome-open &>> /dev/null
            then
                gnome-open "$outfile.html"
            fi
        fi
    else
        pivot_file="./test/perf_out/$now_ts-pivot-snapshot-gas-usages-historical-$(basename "$(dirname "{{path}}")")"
        clipivot max $outfile.csv --rows=test --cols=commit --val=gas > $pivot_file.csv
        echo "Pivot table written to:           $pivot_file.csv"
    fi

alias pl := perf-list
# list all performance files (*gas-usages-*.* and *bytecode-sizes-*.*)
[group('performance')]
perf-list:
    find . -type f \( -name '*-gas-usages-*.*' -o -name '*-bytecode-sizes-*.*' \) -print | sort

alias pr := perf-remove
# remove all performance files (*gas-usages-*.* and *bytecode-sizes-*.*)
[group('performance')]
perf-remove:
    #!/usr/bin/env bash
    files=$(find . -type f \( -name '*-gas-usages-*.*' -o -name '*-bytecode-sizes-*.*' \) -print | sort)

    if [ -z "$files" ]; then
        echo "No performance data files to remove."
        exit 0
    fi

    echo "The following performance data files will be removed:"
    echo "$files"
    echo

    read -r -p 'Do you want to proceed with removing? [y/N] ' yn
    if [[ $yn =~ ^[Yy]$ ]]; then
        echo "Removing..."
        find . -type f \( -name '*-gas-usages-*.*' -o -name '*-bytecode-sizes-*.*' \) -print -delete
    else
        echo "Removing canceled."
    fi

[group('build')]
build-prism:
    cd ./scripts/prism && ./build.sh

[group('build')]
build-highlightjs:
    cd ./scripts/highlightjs && ./build.sh

[group('build')]
generate-sway-lib-std:
    cd ./sway-lib-std && ./generate.sh

[group('test')]
test-forc-fmt-check-panic:
    cd ./scripts/formatter && ./forc-fmt-check-panic.sh