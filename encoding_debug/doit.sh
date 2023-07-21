#!/bin/bash -e

fuel-core run --db-type in-memory &> /dev/null </dev/null &
fuel_core_pid="$!"

contract_id="$( forc deploy --path contract --unsigned | grep 'Contract ID' | rev | cut -d' ' -f1 | rev )"

script_file=script/src/main.sw
backup_file=.backup_script.sw
cp "$script_file" "$backup_file"
sed "s/CONTRACT_ID/$contract_id/g" -i "$script_file"

cleanup() {
    mv "$backup_file" "$script_file" || true
    kill -9 "$fuel_core_pid"
}
trap cleanup EXIT

forc run --path script --contract "$contract_id" --unsigned
