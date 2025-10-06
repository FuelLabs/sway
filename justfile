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

[group('benchmark')]
collect-historic-gas-usage path:
    #! /bin/bash
    mkdir -p target
    rm target/a.csv
    rm target/a.html
    rm target/report.html
    echo "test,gas,hash" > target/a.csv
    for HASH in `git log --format='%H' -- {{path}}`; do
        TIMESTAMP=$(git show -s --format='%as-%ct-%H' "$HASH")
        git --no-pager show "$HASH:{{path}}" | bash -c "scripts/compare-gas-usage/extract-gas-usage.sh $TIMESTAMP" >> target/a.csv
        ./scripts/csv2html/csv2html.sh target/a.csv >> target/a.html
    done
    ./scripts/csv2html/htmltable2report.sh target/a.html > target/report.html

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