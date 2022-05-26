#!/bin/bash
project="highlight.js"
sway="sway.js"

if ! test -d ./${project}; then
    git clone --depth 1 --branch 11.3.1 git@github.com:highlightjs/highlight.js.git
fi

cp ${sway} ${project}/src/languages
cd ${project}
npm ci

rm -rf build
node tools/build.js sway rust ini bash shell json
cp build/highlight.min.js ../../../docs/theme/highlight.js

# add "keep" in order to keep highlight.js repo
if [[ ${1} != "keep" ]]; then
    cd ../
    rm -rf ${project}
fi
