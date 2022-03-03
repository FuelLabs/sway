use crate::{
    cli::BuildCommand,
    lock::Lock,
    pkg,
    utils::helpers::{default_output_directory, lock_path, print_lock_diff, read_manifest},
};
use anyhow::{anyhow, bail, Result};
use std::{
    fs::{self, File},
    io::Write,
    path::PathBuf,
};
use sway_core::source_map::SourceMap;
use sway_utils::{find_manifest_dir, MANIFEST_FILE_NAME};

pub fn build(command: BuildCommand) -> Result<pkg::Compiled> {
    let BuildCommand {
        path,
        binary_outfile,
        use_ir,
        debug_outfile,
        print_finalized_asm,
        print_intermediate_asm,
        print_ir,
        offline_mode: offline,
        silent_mode: silent,
        output_directory,
        minify_json_abi,
    } = command;

    let build_conf = pkg::BuildConf {
        use_ir,
        print_ir,
        print_finalized_asm,
        print_intermediate_asm,
    };

    // find manifest directory, even if in subdirectory
    let this_dir = if let Some(ref path) = path {
        PathBuf::from(path)
    } else {
        std::env::current_dir()?
    };

    let manifest_dir = match find_manifest_dir(&this_dir) {
        Some(dir) => dir,
        None => {
            bail!(
                "could not find `{}` in `{}` or any parent directory",
                MANIFEST_FILE_NAME,
                this_dir.display(),
            );
        }
    };
    let manifest = read_manifest(&manifest_dir)?;
    let lock_path = lock_path(&manifest_dir);

    // Load the build plan from the lock file.
    let plan_result = pkg::BuildPlan::from_lock_file(&lock_path);

    // Retrieve the old lock file state so we can produce a diff.
    let old_lock = plan_result
        .as_ref()
        .ok()
        .map(|plan| Lock::from_graph(&plan.graph))
        .unwrap_or_default();

    // Validate the loaded build plan for the current manifest.
    let plan_result = plan_result.and_then(|plan| plan.validate(&manifest).map(|_| plan));

    // If necessary, construct a new build plan.
    let plan: pkg::BuildPlan = plan_result.or_else(|e| -> Result<pkg::BuildPlan> {
        println!("  Creating a new `Forc.lock` file");
        println!("    Cause: {}", e);
        let plan = pkg::BuildPlan::new(&manifest_dir, offline)?;
        let lock = Lock::from_graph(&plan.graph);
        let diff = lock.diff(&old_lock);
        print_lock_diff(&manifest.project.name, &diff);
        let string = toml::ser::to_string_pretty(&lock)
            .map_err(|e| anyhow!("failed to serialize lock file: {}", e))?;
        fs::write(&lock_path, &string).map_err(|e| anyhow!("failed to write lock file: {}", e))?;
        println!("   Created new lock file at {}", lock_path.display());
        Ok(plan)
    })?;

    // Iterate over and compile all packages.
    let mut namespace_map = Default::default();
    let mut source_map = SourceMap::new();
    let mut json_abi = vec![];
    let mut bytecode = vec![];
    for &node in &plan.compilation_order {
        let dep_namespace =
            pkg::dependency_namespace(&namespace_map, &plan.graph, &plan.compilation_order, node);
        let pkg = &plan.graph[node];
        let path = &plan.path_map[&pkg.id()];
        let res = pkg::compile(
            pkg,
            path,
            &build_conf,
            dep_namespace,
            &mut source_map,
            silent,
        )?;
        let (compiled, maybe_namespace) = res;
        if let Some(namespace) = maybe_namespace {
            namespace_map.insert(node, namespace);
        }
        json_abi.extend(compiled.json_abi);
        bytecode = compiled.bytecode;
        source_map.insert_dependency(path.clone());
    }

    if let Some(outfile) = binary_outfile {
        let mut file = File::create(outfile)?;
        file.write_all(bytecode.as_slice())?;
    }

    if let Some(outfile) = debug_outfile {
        fs::write(
            outfile,
            &serde_json::to_vec(&source_map).expect("JSON serialization failed"),
        )?;
    }

    // TODO: We may support custom build profiles in the future.
    let profile = "debug";

    // Create the output directory for build artifacts.
    let output_dir = output_directory
        .map(PathBuf::from)
        .unwrap_or_else(|| default_output_directory(&manifest_dir).join(profile));
    if !output_dir.exists() {
        fs::create_dir_all(&output_dir)?;
    }

    // Place build artifacts into the output directory.
    let bin_path = output_dir
        .join(&manifest.project.name)
        .with_extension("bin");
    std::fs::write(&bin_path, bytecode.as_slice())?;
    if !json_abi.is_empty() {
        let json_abi_stem = format!("{}-abi", manifest.project.name);
        let json_abi_path = output_dir.join(&json_abi_stem).with_extension("json");
        let file = File::create(json_abi_path)?;
        let res = if minify_json_abi {
            serde_json::to_writer(&file, &json_abi)
        } else {
            serde_json::to_writer_pretty(&file, &json_abi)
        };
        res?;
    }

    println!("  Bytecode size is {} bytes.", bytecode.len());

    Ok(pkg::Compiled { bytecode, json_abi })
}
