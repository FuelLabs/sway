#!/bin/bash

# This script will format all sway projects in the current directory and all subdirectories.
# This is useful for testing the formatter itself to make sure it's not panicking on any valid
# sway projects and for checking that it's formatted output is correct.
forc_manifests=`find . -name Forc.toml`
let count=0
let failed=0
for f in $forc_manifests
do
    dir="${f%/*}"
    forc fmt -p $dir
    if [ $? -ne 0 ]
    then
        echo "Formatting failed: $dir"
        let failed=failed+1
    fi
    let count=count+1
done
echo ""
echo "Failed count: $failed"
echo "Total count: $count"
