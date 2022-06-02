use crate::{cli::BuildCommand, utils::SWAY_GIT_TAG};
use anyhow::{anyhow, bail, Result};
use forc_pkg::{self as pkg, lock, Lock, ManifestFile};
use forc_util::{default_output_directory, lock_path};
use std::{
    fs::{self, File},
    path::PathBuf,
};
use tracing::{info, warn};

pub fn build(command: BuildCommand) -> Result<pkg::Compiled> {
    let BuildCommand {
        path,
        binary_outfile,
        use_orig_asm,
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

    // If any cli parameter is passed by the user it overrides the selected build profile.
    let mut config = &pkg::BuildConfig {
        use_orig_asm,
        print_ir,
        print_finalized_asm,
        print_intermediate_asm,
        silent: silent_mode,
    };

    // Check if any cli parameter is passed by the user if not fetch the build profile from manifest.
    if !use_orig_asm && !print_ir && !print_finalized_asm && !silent_mode {
        config = manifest.build_profiles.as_ref()
        .and_then(|profiles| profiles.get(&selected_build_profile))
        .unwrap_or_else(||{
            warn!("provided profile option {} is not present in the manifest file. Using default config.", selected_build_profile);
            config
        });
    }

    let lock_path = lock_path(manifest.dir());

    // Load the build plan from the lock file.
    let plan_result = pkg::BuildPlan::from_lock_file(&lock_path, SWAY_GIT_TAG);

    // Retrieve the old lock file state so we can produce a diff.
    let old_lock = plan_result
        .as_ref()
        .ok()
        .map(|plan| Lock::from_graph(plan.graph()))
        .unwrap_or_default();

    // Validate the loaded build plan for the current manifest.
    let plan_result =
        plan_result.and_then(|plan| plan.validate(&manifest, SWAY_GIT_TAG).map(|_| plan));

    // If necessary, construct a new build plan.
    let plan: pkg::BuildPlan = plan_result.or_else(|e| -> Result<pkg::BuildPlan> {
        if locked {
            bail!(
                "The lock file {} needs to be updated but --locked was passed to prevent this.",
                lock_path.to_string_lossy()
            );
        }

        let cause = if e.to_string().contains("No such file or directory") {
            anyhow!("lock file did not exist")
        } else {
            e
        };
        info!("  Creating a new `Forc.lock` file. (Cause: {})", cause);
        let plan = pkg::BuildPlan::new(&manifest, SWAY_GIT_TAG, offline)?;
        let lock = Lock::from_graph(plan.graph());
        let diff = lock.diff(&old_lock);
        lock::print_diff(&manifest.project.name, &diff);
        let string = toml::ser::to_string_pretty(&lock)
            .map_err(|e| anyhow!("failed to serialize lock file: {}", e))?;
        fs::write(&lock_path, &string).map_err(|e| anyhow!("failed to write lock file: {}", e))?;
        info!("   Created new lock file at {}", lock_path.display());
        Ok(plan)
    })?;

    // Build it!
    let (compiled, source_map) = pkg::build(&plan, config, SWAY_GIT_TAG)?;

    if let Some(outfile) = binary_outfile {
        fs::write(&outfile, &compiled.bytecode)?;
    }

    if let Some(outfile) = debug_outfile {
        let source_map_json = serde_json::to_vec(&source_map).expect("JSON serialization failed");
        fs::write(outfile, &source_map_json)?;
    }

    // TODO: We may support custom build profiles in the future.
    let profile = "debug";

    // Create the output directory for build artifacts.
    let output_dir = output_directory
        .map(PathBuf::from)
        .unwrap_or_else(|| default_output_directory(manifest.dir()).join(profile));
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

    Ok(compiled)
}
