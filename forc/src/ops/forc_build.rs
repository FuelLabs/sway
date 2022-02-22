use crate::{
    cli::BuildCommand,
    utils::helpers::{default_output_directory, read_manifest},
    pkg,
};
use anyhow::{anyhow, Result};
use std::{
    fs::{self, File},
    io::Write,
    path::PathBuf,
};
use sway_core::{create_module, source_map::SourceMap};
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
        offline_mode,
        silent_mode,
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
            return Err(anyhow!(
                "could not find `{}` in `{}` or any parent directory",
                MANIFEST_FILE_NAME,
                this_dir.display(),
            ))
        }
    };
    let manifest = read_manifest(&manifest_dir)?;

    // Produce the graph of packages.
    // TODO: We should first try to load this from something like a `Forc.lock`.
    let pkg_graph = pkg::fetch_deps(manifest_dir.clone(), &manifest, offline_mode)?;

    // TODO: Warn about duplicate pkg names with differing versions/sources.

    // The `pkg_graph` is of *a -> b* where *a* depends on *b*. We can determine compilation order
    // by performing a toposort of the graph with reversed weights.
    let rev_pkg_graph = petgraph::visit::Reversed(&pkg_graph);
    let compilation_order = petgraph::algo::toposort(rev_pkg_graph, None)
        // TODO: Show full list of packages that cycle.
        .map_err(|e| anyhow!("dependency cycle detected: {:?}", e))?;

    // Iterate over and compile all packages.
    let namespace = create_module();
    let mut source_map = SourceMap::new();
    let mut json_abi = vec![];
    let mut bytecode = vec![];
    for node in compilation_order {
        let pkg = &pkg_graph[node];
        let compiled = pkg::compile(pkg, &build_conf, namespace, &mut source_map, silent_mode)?;
        json_abi.extend(compiled.json_abi);
        bytecode = compiled.bytecode;
        source_map.insert_dependency(pkg.path.clone());
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
