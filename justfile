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
[linux]
compare-gas-usage branchBefore branchAfter:
    #! /bin/bash
    CHANGES=$(git status --porcelain | wc -l)
    if [ "$CHANGES" != "0" ]; then
        echo -e "git is not clean. Aborting."
        exit
    fi
    AFTEROUT=$(sed "s/\//-/g" <<< "{{branchAfter}}")
    AFTEROUT="target/gas-$AFTEROUT.txt"
    git checkout {{branchAfter}}
    cargo r -p test --release -- --verbose --forc-test-only | tee /dev/tty | ./scripts/compare-gas-usage/extract-gas-usage.sh > "$AFTEROUT"

    BEFOREOUT=$(sed "s/\//-/g" <<< "{{branchBefore}}")
    BEFOREOUT="target/gas-$BEFOREOUT.txt"
    git checkout {{branchBefore}}
    cargo r -p test --release -- --verbose --forc-test-only | tee /dev/tty | ./scripts/compare-gas-usage/extract-gas-usage.sh > "$BEFOREOUT"

    scripts/compare-gas-usage/generate-diff.sh "$BEFOREOUT" "$AFTEROUT"

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