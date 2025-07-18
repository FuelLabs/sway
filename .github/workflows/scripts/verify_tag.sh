#!/usr/bin/env bash
set -e

err() {
    echo -e "\e[31m\e[1merror:\e[0m $@" 1>&2;
}

status() {
    local width=12
    printf "\e[32m\e[1m%${width}s\e[0m %s\n" "$1" "$2"
}

get_toml_version () {
    local toml_path="$1"

    local manifest="Cargo.toml"
    echo $(toml get $manifest $toml_path | tr -d '"')
}

check_version () {
    local ref=$1
    local toml_path=$2

    # strip preceding 'v' if it exists on tag
    ref=${ref/#v}

    local toml_version=$(get_toml_version "$toml_path")
  
    if [ "$toml_version" != "$ref" ]; then
        err "Crate version $toml_version for $toml_path, doesn't match tag version $ref"
        exit 1
    else
      status "Crate version for $toml_path matches tag $toml_version"
    fi
}

REF=$1

if [ -z "$REF" ]; then
    err "Expected ref to be set"
    exit 1
fi

for toml_path in \
    "workspace.package.version" \
    "workspace.dependencies.forc.version" \
    "workspace.dependencies.forc-pkg.version" \
    "workspace.dependencies.forc-test.version" \
    "workspace.dependencies.forc-tracing.version" \
    "workspace.dependencies.forc-util.version" \
    "workspace.dependencies.forc-plugins.version" \
    "workspace.dependencies.forc-client.version" \
    "workspace.dependencies.forc-crypto.version" \
    "workspace.dependencies.forc-debug.version" \
    "workspace.dependencies.forc-doc.version" \
    "workspace.dependencies.forc-fmt.version" \
    "workspace.dependencies.forc-lsp.version" \
    "workspace.dependencies.forc-tx.version" \
    "workspace.dependencies.forc-migrate.version" \
    "workspace.dependencies.forc-publish.version" \
    "workspace.dependencies.sway-ast.version" \
    "workspace.dependencies.sway-core.version" \
    "workspace.dependencies.sway-error.version" \
    "workspace.dependencies.sway-lsp.version" \
    "workspace.dependencies.sway-parse.version" \
    "workspace.dependencies.sway-types.version" \
    "workspace.dependencies.sway-utils.version" \
    "workspace.dependencies.swayfmt.version" \
    "workspace.dependencies.sway-ir.version" \
    "workspace.dependencies.sway-ir-macros.version" \
; do
    check_version $REF $toml_path
done
