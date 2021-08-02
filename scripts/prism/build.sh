#!/bin/bash
project="prism"
components="components.json"
sway="prism-sway.js"

if ! test -d ./${project}; then
    git clone git@github.com:PrismJS/prism.git
fi

cp ${sway} ${project}/components
cp ${components} ${project}
cd ${project}
npm ci

npm run build
cp components/prism-sway.min.js ../prism-sway.min.js

# add "keep" in order to keep highlight.js repo
if [[ ${1} != "keep" ]]; then
    cd ../
    rm -rf ${project}
fi
