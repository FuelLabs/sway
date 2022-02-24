use crate::{
    cli::BuildCommand,
    lock::Lock,
    pkg,
    utils::helpers::{default_output_directory, lock_path, read_manifest},
};
use anyhow::{anyhow, bail, Result};
use std::{
    fs::{self, File},
    io::Write,
    path::{Path, PathBuf},
};
use sway_core::{create_module, source_map::SourceMap};
use sway_utils::{find_manifest_dir, MANIFEST_FILE_NAME};

struct BuildPlan {
    pkg_graph: pkg::Graph,
    pkg_path_map: pkg::PathMap,
    compilation_order: Vec<pkg::NodeIx>,
}

impl BuildPlan {
    /// Create a new build plan for the project by fetching and pinning dependenies.
    pub fn new(manifest_dir: &Path, offline: bool) -> Result<Self> {
        let manifest = read_manifest(manifest_dir)?;
        let (graph, path_map) = pkg::fetch_deps(manifest_dir.to_path_buf(), &manifest, offline)?;
        let compilation_order = pkg::compilation_order(&graph)?;
        Ok(Self {
            pkg_graph: graph,
            pkg_path_map: path_map,
            compilation_order,
        })
    }

    /// Attempt to load the build plan from the `Forc.lock` file.
    pub fn from_lock_file(lock_path: &Path) -> Result<Self> {
        let proj_path = lock_path.parent().unwrap();
        let lock = Lock::from_path(lock_path)?;
        let graph = lock.to_graph()?;
        let compilation_order = pkg::compilation_order(&graph)?;
        let path_map = pkg::graph_to_path_map(proj_path, &graph, &compilation_order)?;
        Ok(Self {
            pkg_graph: graph,
            pkg_path_map: path_map,
            compilation_order,
        })
    }
}

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
        std::env::current_dir().map_err(|e| anyhow!("{:?}", e))?
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

    // Attempt to load the build plan or otherwise create a new one.
    let lock_path = lock_path(&manifest_dir);
    let plan = match BuildPlan::from_lock_file(&lock_path) {
        Ok(plan) => plan,
        Err(e) => {
            println!("Unable to create build plan from lock file: {}", e);
            println!("Updating build plan and lock file...");
            let plan = BuildPlan::new(&manifest_dir, offline)?;
            let lock = Lock::from_graph(&plan.pkg_graph);
            let string = toml::ser::to_string_pretty(&lock)
                .map_err(|e| anyhow!("failed to serialize lock file: {}", e))?;
            fs::write(&lock_path, &string)
                .map_err(|e| anyhow!("failed to write lock file: {}", e))?;
            println!("Updated `Forc.lock` written to {}", lock_path.display());
            plan
        }
    };

    // Iterate over and compile all packages.
    let namespace = create_module();
    let mut source_map = SourceMap::new();
    let mut json_abi = vec![];
    let mut bytecode = vec![];
    for node in plan.compilation_order {
        let pkg = &plan.pkg_graph[node];
        let path = &plan.pkg_path_map[&pkg.id()];
        let compiled = pkg::compile(pkg, path, &build_conf, namespace, &mut source_map, silent)?;
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
        )
        .map_err(|e| e)?;
    }

    // TODO: We may support custom build profiles in the future.
    let profile = "debug";

    // Create the output directory for build artifacts.
    let output_dir = output_directory
        .map(PathBuf::from)
        .unwrap_or_else(|| default_output_directory(&manifest_dir).join(profile));
    if !output_dir.exists() {
        fs::create_dir_all(&output_dir).map_err(|e| e)?;
    }

    // Place build artifacts into the output directory.
    let bin_path = output_dir
        .join(&manifest.project.name)
        .with_extension("bin");
    std::fs::write(&bin_path, bytecode.as_slice())?;
    if !json_abi.is_empty() {
        let json_abi_stem = format!("{}-abi", manifest.project.name);
        let json_abi_path = output_dir.join(&json_abi_stem).with_extension("json");
        let file = File::create(json_abi_path).map_err(|e| e)?;
        let res = if minify_json_abi {
            serde_json::to_writer(&file, &json_abi)
        } else {
            serde_json::to_writer_pretty(&file, &json_abi)
        };
        res.map_err(|e| e)?;
    }

    println!("  Bytecode size is {} bytes.", bytecode.len());

    Ok(pkg::Compiled { bytecode, json_abi })
}
