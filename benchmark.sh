#!/usr/bin/env bash

SCRIPT_DIR=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )

PREPARE_FOR_COMMIT=false
while [[ $# -gt 0 ]]
do
    key="$1"
    case $key in
        --prepare-for-commit)
            PREPARE_FOR_COMMIT=true
            shift # past argument
            ;;
        *)
        shift # past argument
        ;;
    esac
done

benchmarks_dir="benchmarks"
if [ ! -d "$benchmarks_dir" ]; then
    mkdir -p "$benchmarks_dir"
fi

if [ -d "$SCRIPT_DIR/target/release" ]; then
    build_type="release"
elif [ -d "$SCRIPT_DIR/target/debug" ]; then
    build_type="debug"
else
    echo "Neither target/release nor target/debug directories found. Exiting..."
    exit 1
fi

forc_path="$SCRIPT_DIR/target/$build_type/forc"

# prepare the benchmark data for commit if requested
if $PREPARE_FOR_COMMIT; then
    sway_peformance_data_dir=performance-data
    sway_performance_data_repo_url=git@github.com:FuelLabs/sway-performance-data.git

    if [ ! -d "$SCRIPT_DIR/$sway_peformance_data_dir" ]; then
        echo "Directory $sway_peformance_data_dir not found. Cloning the repository..."
        git clone "$sway_performance_data_repo_url" "$sway_peformance_data_dir"
        echo "Repository cloned into $sway_peformance_data_dir."
    else
        echo "Updating sway-performance-data repository..."
        git -C "$SCRIPT_DIR/$sway_peformance_data_dir" pull
    fi

    mkdir -p "$SCRIPT_DIR/$sway_peformance_data_dir/$GITHUB_SHA"
    cp -r $benchmarks_dir/* "$SCRIPT_DIR/$sway_peformance_data_dir/$GITHUB_SHA"
else
    sway_libs_dir=sway-libs
    sway_libs_repo_url=https://github.com/FuelLabs/sway-libs.git
    sway_libs_branch_name="benchmarks"

    if [ ! -d "$SCRIPT_DIR/$sway_libs_dir" ]; then
        echo "Directory $sway_libs_dir not found. Cloning the repository..."
        git clone -b "$sway_libs_branch_name" "$sway_libs_repo_url" "$sway_libs_dir"
        echo "Repository cloned with branch $sway_libs_branch_name into $sway_libs_dir."
    fi

    libs=(
        "fixed_point"
        "merkle_proof"
        "nft"
        "ownership"
        "reentrancy"
        "signed_integers"
        "storagemapvec"
        "strings/storage_string"
        "strings/string"
    )

    sway_apps_dir=sway-applications
    sway_apps_repo_url=https://github.com/FuelLabs/sway-applications.git
    sway_apps_branch_name="master"

    if [ ! -d "$SCRIPT_DIR/$sway_apps_dir" ]; then
        echo "Directory $sway_apps_dir not found. Cloning the repository..."
        git clone -b "$sway_apps_branch_name" "$sway_apps_repo_url" "$sway_apps_dir"
        echo "Repository cloned with branch $sway_apps_branch_name into $sway_apps_dir."
    fi

    sway_libs_revision=$(git -C $SCRIPT_DIR/$sway_libs_dir rev-parse HEAD)
    sway_apps_revision=$(git -C $SCRIPT_DIR/$sway_apps_dir rev-parse HEAD)
    sway_git_revision=$(git rev-parse HEAD)

    for lib in "${libs[@]}"; do
        echo "Benchmarking $lib..."
        project_name=$(basename "$lib")
        metrics_json_file="$benchmarks_dir/$project_name.json"
        output=$(/usr/bin/time -f '{"elapsed": "%e", "cpu_usage": "%P", "memory": "%MKB"}' \
            $forc_path build --path "$SCRIPT_DIR/sway-libs/libs/$lib" \
            --metrics-outfile="$metrics_json_file" 2>&1)

        exit_status=$?
        if [ $exit_status -ne 0 ]; then
            echo "  Failed, ignoring."
            continue
        fi

        # filter out forc warnings by only matching on the JSON metrics data
        json_stats=$(echo "$output" | grep -Eo '^\s*{[^}]*}')

        metrics_json=$(cat "$metrics_json_file" | jq '{phases: .}')
        merged_json=$(jq -s '.[0] * .[1]' <(echo "$metrics_json") <(echo "$json_stats"))
        merged_json=$(jq --arg bt "$build_type" '. + {build_type: $bt}' <<< "$merged_json")
        merged_json=$(jq --arg gr "$sway_apps_revision" '. + {sway_apps_revision: $gr}' <<< "$merged_json")
        merged_json=$(jq --arg gr "$sway_libs_revision" '. + {sway_libs_revision: $gr}' <<< "$merged_json")
        merged_json=$(jq --arg gr "$sway_git_revision" '. + {sway_git_revision: $gr}' <<< "$merged_json")

        echo "$merged_json" > $metrics_json_file
    done
fi
