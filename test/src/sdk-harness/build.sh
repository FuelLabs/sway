#!/usr/bin/env bash

# Pass `--locked` when running this script in CI to ensure both cargo and forc
# lock files are up to date.
locked="$1"

# Cross platform version of `realpath` or `readlink`.
abs_path() {
  (cd "$1"; pwd)
}

# Grab the absolute path to this script.
base_dir="$(abs_path $(dirname $0))"

# Search for the parent Cargo manifest for Forc.
parent_manifest_dir="${base_dir}"
while true; do
  parent_manifest_dir=$(abs_path "${parent_manifest_dir}/..")
  if [[ -f "${parent_manifest_dir}/Cargo.toml" ]]; then
    forc="cargo run $locked --manifest-path ${parent_manifest_dir}/Cargo.toml --package forc --"
    break
  fi
  if [[ "${parent_manifest_dir}" = "/" ]]; then
    # Not found for some reason.  Default to an installed binary.
    forc="forc"
    break
  fi
done

test_dirs="${base_dir}/test_artifacts/* ${base_dir}/test_projects/* ${base_dir}/test_artifacts/*/*"

for test_dir in $test_dirs; do
  if [[ -f "${test_dir}/Forc.toml" ]]; then
    echo "Building test $test_dir..."
    # This can be done in a better way, but I'm not going to invest time in it
    # because it is temporary
    if [[ ${test_dir} == *"experimental_storage"* ]]; then
        ${forc} build $locked --experimental-storage -o temp -p "${test_dir}" && echo ✔
    else
        echo ✔
        ${forc} build $locked -o temp -p "${test_dir}" && echo ✔
    fi
    if ! [[ -f temp ]]; then
      echo  "❌  Failed to build $test_dir"
      exit 1
    fi
    rm -f temp
  else
    echo "Skipping test $test_dir..."
  fi
done

echo "Successfully built all projects."
