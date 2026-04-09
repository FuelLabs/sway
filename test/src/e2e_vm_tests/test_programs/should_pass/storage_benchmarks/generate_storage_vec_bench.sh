#!/usr/bin/env bash
#
# Generates a storage_vec benchmark project for a given element size.
#
# Usage:
#   ./generate_storage_vec_bench.sh <size>
#   ./generate_storage_vec_bench.sh all       # generate all sizes
#
# Sizes: 8, 24, 32, 56, 72, 88, 96
#
# The generated project directory is placed alongside this script as:
#   storage_vec_s<size>/
#
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

ALL_SIZES=(8 24 32 56 72 88 96)
COUNTS=(10 100 1000)
OPS=(push push_n_elems_into_empty_vec pop get set first last len is_empty swap swap_remove remove insert reverse fill resize_grow resize_shrink store_vec load_vec iter clear)

# ── Size → type mapping ────────────────────────────────────────────

type_for_size() {
    case "$1" in
        8)  echo "u64" ;;
        *)  echo "Struct${1}" ;;
    esac
}

default_for_size() {
    case "$1" in
        8)  echo "0" ;;
        *)  echo "STRUCT${1}_DEFAULT" ;;
    esac
}

needs_stored_types() {
    [[ "$1" != "8" ]]
}

# ── Helpers ─────────────────────────────────────────────────────────

# Emit the populate loop (push N elements into storage.vec)
emit_populate() {
    local n="$1" default="$2"
    cat <<SWAY
        let mut i = 0;
        while i < ${n} {
            storage.vec.push(${default});
            i += 1;
        }
SWAY
}

# Emit the heap-vec build loop (build Vec<T> of N elements on the heap)
emit_vec_build() {
    local n="$1" default="$2" type="$3"
    cat <<SWAY
        let mut v = Vec::<${type}>::new();
        let mut i = 0;
        while i < ${n} {
            v.push(${default});
            i += 1;
        }
SWAY
}

# Emit the operation-specific line(s) after the populate setup
emit_op_tail() {
    local op="$1" n="$2" default="$3"
    local mid=$((n / 2))
    local last=$((n - 1))
    local double=$((n * 2))
    local half=$((n / 2))

    case "$op" in
        push)          echo "        storage.vec.push(${default});" ;;
        pop)           echo "        let _ = storage.vec.pop();" ;;
        get)           echo "        let _ = storage.vec.get(${mid}).unwrap().try_read();" ;;
        set)           echo "        storage.vec.set(${mid}, ${default});" ;;
        first)         echo "        let _ = storage.vec.first().unwrap().try_read();" ;;
        last)          echo "        let _ = storage.vec.last().unwrap().try_read();" ;;
        len)           echo "        let _ = storage.vec.len();" ;;
        is_empty)      echo "        let _ = storage.vec.is_empty();" ;;
        swap)          echo "        storage.vec.swap(0, ${last});" ;;
        swap_remove)   echo "        let _ = storage.vec.swap_remove(${mid});" ;;
        remove)        echo "        let _ = storage.vec.remove(${mid});" ;;
        insert)        echo "        storage.vec.insert(${mid}, ${default});" ;;
        reverse)       echo "        storage.vec.reverse();" ;;
        fill)          echo "        storage.vec.fill(${default});" ;;
        resize_grow)   echo "        storage.vec.resize(${double}, ${default});" ;;
        resize_shrink) echo "        storage.vec.resize(${half}, ${default});" ;;
        load_vec)      echo "        let _ = storage.vec.load_vec();" ;;
        iter)
            echo "        for elem in storage.vec.iter() {"
            echo "            let _ = elem.try_read();"
            echo "        }"
            ;;
        clear)         echo "        let _ = storage.vec.clear();" ;;
    esac
}

# ── Generate one project ───────────────────────────────────────────

generate_project() {
    local size="$1"
    local type default abi_name project_name project_dir
    type=$(type_for_size "$size")
    default=$(default_for_size "$size")
    project_name="storage_vec_s${size}"
    project_dir="$SCRIPT_DIR/$project_name"
    abi_name="StorageVecS${size}Abi"

    mkdir -p "$project_dir/src"

    # ── Forc.toml ───────────────────────────────────────────────────
    {
        cat <<EOF
[project]
authors = ["Fuel Labs <contact@fuel.sh>"]
entry = "main.sw"
license = "Apache-2.0"
name = "${project_name}"

[dependencies]
std = { path = "../../../../../../../sway-lib-std" }
EOF
        if needs_stored_types "$size"; then
            echo 'stored_types = { path = "../stored_types" }'
        fi
    } > "$project_dir/Forc.toml"

    # ── test.toml ───────────────────────────────────────────────────
    echo 'category = "unit_tests_pass"' > "$project_dir/test.toml"

    # ── src/main.sw ─────────────────────────────────────────────────
    {
        echo "contract;"
        echo ""
        if needs_stored_types "$size"; then
            echo "use stored_types::*;"
        fi
        echo "use std::storage::storage_vec::*;"
        echo ""
        echo "storage {"
        echo "    vec: StorageVec<${type}> = StorageVec {},"
        echo "}"
        echo ""
        echo "impl Contract {"

        # ── Empty-call baseline ──────────────────────────────────────
        echo ""
        echo "    // === Baseline (empty contract method call) ==="
        echo ""
        echo "    fn baseline() { }"

        # ── Populate baselines ──────────────────────────────────────
        echo ""
        echo "    // === Baselines (populate N elements) ==="
        for n in "${COUNTS[@]}"; do
            echo ""
            echo "    #[storage(read, write)]"
            echo "    fn baseline_n${n}() {"
            emit_populate "$n" "$default"
            echo "    }"
        done

        # ── store_vec baselines ─────────────────────────────────────
        echo ""
        echo "    // === Baselines (build heap Vec of N elements) ==="
        for n in "${COUNTS[@]}"; do
            echo ""
            echo "    fn baseline_store_vec_n${n}() {"
            emit_vec_build "$n" "$default" "$type"
            echo "    }"
        done

        # ── Operation methods ───────────────────────────────────────
        for op in "${OPS[@]}"; do
            echo ""
            echo "    // === ${op} ==="
            for n in "${COUNTS[@]}"; do
                echo ""
                if [[ "$op" == "store_vec" ]]; then
                    echo "    #[storage(write)]"
                    echo "    fn ${op}_n${n}() {"
                    emit_vec_build "$n" "$default" "$type"
                    echo "        storage.vec.store_vec(v);"
                elif [[ "$op" == "push_n_elems_into_empty_vec" ]]; then
                    echo "    #[storage(read, write)]"
                    echo "    fn ${op}_n${n}() {"
                    emit_populate "$n" "$default"
                else
                    echo "    #[storage(read, write)]"
                    echo "    fn ${op}_n${n}() {"
                    emit_populate "$n" "$default"
                    emit_op_tail "$op" "$n" "$default"
                fi
                echo "    }"
            done
        done

        echo "}"
        echo ""

        # ── Test functions ──────────────────────────────────────────
        echo "// === Baseline test (empty call) ==="
        echo ""
        echo "#[test]"
        echo "fn bench_baseline() {"
        echo "    let caller = abi(${abi_name}, CONTRACT_ID);"
        echo "    caller.baseline();"
        echo "}"
        echo ""

        echo "// === Baseline tests (populate) ==="
        echo ""
        for n in "${COUNTS[@]}"; do
            echo "#[test]"
            echo "fn bench_baseline_n${n}() {"
            echo "    let caller = abi(${abi_name}, CONTRACT_ID);"
            echo "    caller.baseline_n${n}();"
            echo "}"
            echo ""
        done

        echo "// === Baseline tests (store_vec) ==="
        echo ""
        for n in "${COUNTS[@]}"; do
            echo "#[test]"
            echo "fn bench_baseline_store_vec_n${n}() {"
            echo "    let caller = abi(${abi_name}, CONTRACT_ID);"
            echo "    caller.baseline_store_vec_n${n}();"
            echo "}"
            echo ""
        done

        for op in "${OPS[@]}"; do
            echo "// === ${op} tests ==="
            echo ""
            for n in "${COUNTS[@]}"; do
                echo "#[test]"
                echo "fn bench_${op}_n${n}() {"
                echo "    let caller = abi(${abi_name}, CONTRACT_ID);"
                echo "    caller.${op}_n${n}();"
                echo "}"
                echo ""
            done
        done

    } > "$project_dir/src/main.sw"

    echo "Generated: $project_dir"
}

# ── Main ────────────────────────────────────────────────────────────

if [[ $# -eq 0 ]] || [[ "$1" == "all" ]]; then
    for size in "${ALL_SIZES[@]}"; do
        generate_project "$size"
    done
else
    for size in "$@"; do
        generate_project "$size"
    done
fi
