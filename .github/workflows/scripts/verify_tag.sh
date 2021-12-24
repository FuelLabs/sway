#!/usr/bin/env bash
set -e

err() {
    echo -e "\e[31m\e[1merror:\e[0m $@" 1>&2;
}

status() {
    WIDTH=12
    printf "\e[32m\e[1m%${WIDTH}s\e[0m %s\n" "$1" "$2"
}

REF=$1
MANIFEST=$2

if [ -z "$REF" ]; then
    err "Expected ref to be set"
    exit 1
fi

if [ -z "$MANIFEST" ]; then
    err "Expected manifest to be set"
    exit 1
fi

# strip preceeding 'v' if it exists on tag
REF=${REF/#v}
TOML_VERSION=$(toml get $MANIFEST package.version | tr -d '"')

if [ "$TOML_VERSION" != "$REF" ]; then
    err "Crate version $TOML_VERSION, doesn't match tag version $REF"
    exit 1
else
  status "Crate version matches tag $TOML_VERSION"
fi
