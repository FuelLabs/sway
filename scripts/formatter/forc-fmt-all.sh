#!/bin/bash

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


