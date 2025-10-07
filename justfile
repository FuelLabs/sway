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

[group('benchmark')]
benchmark:
    bash ./benchmark.sh

[group('benchmark')]
benchmark-tests:
    bash ./test/bench.sh

[group('benchmark')]
collect-gas-usage:
    cargo r -p test --release -- --verbose --forc-test-only | ./scripts/compare-gas-usage/extract-gas-usage.sh

# This recipe should be used on snapshot tests that contains gas usage from `forc test`,
# because it will extract gas usage from all versions of the file
# revision_range: as used in git to select the verions of the file that gas will be extracted
# path: path to file to extract gas usage
# report: csv or html
# open: for "html", "-o" will open the report in the default browser
[linux]
[group('benchmark')]
collect-historic-gas-usage revision_range path report open="":
    #! /bin/bash
    mkdir -p target
    rm target/a.csv &>> /dev/null
    rm target/a.html &>> /dev/null
    echo "test,gas,commit" > target/a.csv
    for HASH in `git log --format='%H' {{revision_range}} -- {{path}}`; do
        TIMESTAMP=$(git show -s --format='%as-%ct-%H' "$HASH")
        git --no-pager show "$HASH:{{path}}" | bash -c "scripts/compare-gas-usage/extract-gas-usage.sh $TIMESTAMP" >> target/a.csv
    done

    if [ "{{report}}" = "html" ]; then
        ./scripts/csv2html/csv2html.sh target/a.csv >> target/a.html
        if [ -n "{{open}}" ]; then
            if which xdg-open &>> /dev/null
            then
                xdg-open target/a.html
            elif which gnome-open &>> /dev/null
            then
                gnome-open target/a.html
            fi
        fi
    else
        clipivot max target/a.csv --rows=test --cols=commit --val=gas > target/b.csv
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