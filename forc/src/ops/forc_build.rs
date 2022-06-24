use crate::{
    cli::BuildCommand,
    utils::{SWAY_BIN_HASH_SUFFIX, SWAY_BIN_ROOT_SUFFIX, SWAY_GIT_TAG},
};
use anyhow::Result;
use forc_pkg::{self as pkg, ManifestFile};
use forc_util::default_output_directory;
use fuel_tx::Contract;
use std::{
    fs::{self, File},
    path::PathBuf,
};
use sway_core::TreeType;
use tracing::{info, warn};

pub fn build(command: BuildCommand) -> Result<pkg::Compiled> {
    let BuildCommand {
        path,
        binary_outfile,
        debug_outfile,
        print_finalized_asm,
        print_intermediate_asm,
        print_ir,
        offline_mode: offline,
        silent_mode,
        output_directory,
        minify_json_abi,
        locked,
        build_profile,
        release,
        time_phases,
    } = command;

    let key_debug: String = "debug".to_string();
    let key_release: String = "release".to_string();

    let mut selected_build_profile = key_debug;
    if build_profile.is_none() && release {
        selected_build_profile = key_release;
    } else if build_profile.is_some() && release {
        // Here build_profile is guaranteed to be a value.
        warn!(
            "Both {} and release provided as build profile. Using release!",
            build_profile.unwrap()
        );
        selected_build_profile = key_release;
    } else if let Some(build_profile) = build_profile {
        // Here build_profile is guaranteed to be a value.
        selected_build_profile = build_profile;
    }

    let this_dir = if let Some(ref path) = path {
        PathBuf::from(path)
    } else {
        std::env::current_dir()?
    };

    let manifest = ManifestFile::from_dir(&this_dir, SWAY_GIT_TAG)?;

    let plan = pkg::BuildPlan::load_from_manifest(&manifest, locked, offline, SWAY_GIT_TAG)?;

    // Retrieve the specified build profile
    let mut profile = manifest
        .build_profile(&selected_build_profile)
        .cloned()
        .unwrap_or_else(|| {
            warn!(
                "provided profile option {} is not present in the manifest file. \
            Using default profile.",
                selected_build_profile
            );
            Default::default()
        });
    profile.print_ir |= print_ir;
    profile.print_finalized_asm |= print_finalized_asm;
    profile.print_intermediate_asm |= print_intermediate_asm;
    profile.silent |= silent_mode;
    profile.time_phases |= time_phases;

    // Build it!
    let (compiled, source_map) = pkg::build(&plan, &profile, SWAY_GIT_TAG)?;

    if let Some(outfile) = binary_outfile {
        fs::write(&outfile, &compiled.bytecode)?;
    }

    if let Some(outfile) = debug_outfile {
        let source_map_json = serde_json::to_vec(&source_map).expect("JSON serialization failed");
        fs::write(outfile, &source_map_json)?;
    }

    // Create the output directory for build artifacts.
    let output_dir = output_directory
        .map(PathBuf::from)
        .unwrap_or_else(|| default_output_directory(manifest.dir()).join(selected_build_profile));
    if !output_dir.exists() {
        fs::create_dir_all(&output_dir)?;
    }

    // Place build artifacts into the output directory.
    let bin_path = output_dir
        .join(&manifest.project.name)
        .with_extension("bin");
    fs::write(&bin_path, &compiled.bytecode)?;
    if !compiled.json_abi.is_empty() {
        let json_abi_stem = format!("{}-abi", manifest.project.name);
        let json_abi_path = output_dir.join(&json_abi_stem).with_extension("json");
        let file = File::create(json_abi_path)?;
        let res = if minify_json_abi {
            serde_json::to_writer(&file, &compiled.json_abi)
        } else {
            serde_json::to_writer_pretty(&file, &compiled.json_abi)
        };
        res?;
    }

    info!("  Bytecode size is {} bytes.", compiled.bytecode.len());

    match compiled.tree_type {
        TreeType::Script => {
            // hash the bytecode for scripts and store the result in a file in the output directory
            let bytecode_hash = format!("0x{}", fuel_crypto::Hasher::hash(&compiled.bytecode));
            let hash_file_name = format!("{}{}", &manifest.project.name, SWAY_BIN_HASH_SUFFIX);
            let hash_path = output_dir.join(hash_file_name);
            fs::write(hash_path, &bytecode_hash)?;
            info!("  Script bytecode hash: {}", bytecode_hash);
        }
        TreeType::Predicate => {
            // get the root hash of the bytecode for predicates and store the result in a file in the output directory
            let root = format!("0x{}", Contract::root_from_code(&compiled.bytecode));
            let root_file_name = format!("{}{}", &manifest.project.name, SWAY_BIN_ROOT_SUFFIX);
            let root_path = output_dir.join(root_file_name);
            fs::write(root_path, &root)?;
            info!("  Predicate root: {}", root);
        }
        _ => (),
    }

    Ok(compiled)
}
