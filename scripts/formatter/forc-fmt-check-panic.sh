#!/bin/bash

# This script will format all sway projects in the current directory and all subdirectories.
# This is useful for testing the formatter itself to make sure it's not panicking on any valid
# sway projects and for checking that it's formatted output is correct.
forc_manifests=`find . -name Forc.toml`
let count=0
let panicked=0
for f in $forc_manifests
do
    dir="${f%/*}"
    stderr="$(forc fmt -p $dir 2>&1 > /dev/null)"
    if [[ $stderr == *"panicked at"* ]]
    then
        echo ""
        echo "Formatter panicked: $dir"
        let panicked=panicked+1
        echo $stderr
    fi
    let count=count+1
done
echo ""
echo "Panicked count: $panicked"
echo "Total count: $count"

if [[ $panicked -gt 0 ]]
then
    exit 1
fi
