#!/bin/bash

# Run this script to only bump fuel maintained dependencies. 
#
# We currently pin dependencies using "X.Y" in the root Cargo.toml file.
# Since `cargo build` does not check for minor version bumps at each invocation
# it is hard to move between new versions, especially for minor bumps which
# happens much often. Use this script to keep every fuel owned dependency up to
# date.

# Define the list of fuel maintained crates
crates=(
    "fuel-abi-types"
    "fuel-core-client"
    "fuel-core-types"
    "fuels"
    "fuels-core"
    "fuels-accounts"
    "fuel-asm"
    "fuel-crypto"
    "fuel-types"
    "fuel-tx"
    "fuel-vm"
    "forc-wallet"
)

# Run `cargo update -p <crate_name>` for each fuel owned crate. 
for crate in "${crates[@]}"; do
    echo "Updating package: $crate"
    cargo update -p "$crate"
done
