use crate::utils::dependency::{Dependency, DependencyDetails};
use crate::utils::helpers::{find_file_name, find_main_path};
use crate::{
    cli::BuildCommand,
    utils::dependency,
    utils::helpers::{
        get_main_file, print_on_failure, print_on_success, print_on_success_library, read_manifest,
    },
};
use std::fs::{self, File};
use std::io::Write;
use std::sync::Arc;
use sway_core::{FinalizedAsm, TreeType};
use sway_utils::{constants, find_manifest_dir};

use sway_core::{
    create_module, source_map::SourceMap, BuildConfig, BytecodeCompilationResult,
    CompilationResult, NamespaceRef, NamespaceWrapper,
};

use anyhow::Result;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

pub fn build(command: BuildCommand) -> Result<Vec<u8>, String> {
    // find manifest directory, even if in subdirectory
    let this_dir = if let Some(ref path) = command.path {
        PathBuf::from(path)
    } else {
        std::env::current_dir().map_err(|e| format!("{:?}", e))?
    };

    let BuildCommand {
        binary_outfile,
        use_ir,
        debug_outfile,
        print_finalized_asm,
        print_intermediate_asm,
        print_ir,
        offline_mode,
        silent_mode,
        ..
    } = command;
    let manifest_dir = match find_manifest_dir(&this_dir) {
        Some(dir) => dir,
        None => {
            return Err(format!(
                "No manifest file found in this directory or any parent directories of it: {:?}",
                this_dir
            ))
        }
    };

    let mut manifest = read_manifest(&manifest_dir)?;

    let main_path = {
        let mut code_dir = manifest_dir.clone();
        code_dir.push(constants::SRC_DIR);
        code_dir.push(&manifest.project.entry);
        code_dir
    };
    let mut file_path = manifest_dir.clone();
    file_path.pop();
    let file_name = match main_path.strip_prefix(file_path.clone()) {
        Ok(o) => o,
        Err(err) => return Err(err.to_string()),
    };

    let build_config = BuildConfig::root_from_file_name_and_manifest_path(
        file_name.to_path_buf(),
        manifest_dir.clone(),
    )
    .use_ir(use_ir || print_ir) // --print-ir implies --use-ir.
    .print_finalized_asm(print_finalized_asm)
    .print_intermediate_asm(print_intermediate_asm)
    .print_ir(print_ir);

    let mut dependency_graph = HashMap::new();
    let namespace = create_module();

    if let Some(ref mut deps) = manifest.dependencies {
        for (dependency_name, dependency_details) in deps.iter_mut() {
            compile_dependency_lib(
                &this_dir,
                dependency_name,
                dependency_details,
                namespace,
                &mut dependency_graph,
                silent_mode,
                offline_mode,
            )?;
        }
    }

    // now, compile this program with all of its dependencies
    let main_file = get_main_file(&manifest, &manifest_dir)?;

    let mut source_map = SourceMap::new();

    let main = compile(
        main_file,
        &manifest.project.name,
        namespace,
        build_config,
        &mut dependency_graph,
        &mut source_map,
        silent_mode,
    )?;

    if let Some(outfile) = binary_outfile {
        let mut file = File::create(outfile).map_err(|e| e.to_string())?;
        file.write_all(main.as_slice()).map_err(|e| e.to_string())?;
    }

    if let Some(outfile) = debug_outfile {
        fs::write(
            outfile,
            &serde_json::to_vec(&source_map).expect("JSON seralizatio failed"),
        )
        .map_err(|e| e.to_string())?;
    }

    println!("  Bytecode size is {} bytes.", main.len());

    Ok(main)
}

/// Takes a dependency and returns a namespace of exported things from that dependency
/// trait implementations are included as well
fn compile_dependency_lib<'manifest>(
    project_file_path: &Path,
    dependency_name: &'manifest str,
    dependency_lib: &mut Dependency,
    namespace: NamespaceRef,
    dependency_graph: &mut HashMap<String, HashSet<String>>,
    silent_mode: bool,
    offline_mode: bool,
) -> Result<(), String> {
    let mut details = match dependency_lib {
        Dependency::Simple(..) => {
            return Err(
                "Not yet implemented: Simple version-spec dependencies require a registry.".into(),
            )
        }
        Dependency::Detailed(ref mut details) => details,
    };
    // Download a non-local dependency if the `git` property is set in this dependency.
    if let Some(ref git) = details.git {
        // the qualified name of the dependency includes its source and some metadata to prevent
        // conflating dependencies from different sources
        let fully_qualified_dep_name = format!("{}-{}", dependency_name, git);
        let downloaded_dep_path = match dependency::download_github_dep(
            &fully_qualified_dep_name,
            git,
            &details.branch,
            &details.version,
            offline_mode.into(),
        ) {
            Ok(path) => path,
            Err(e) => {
                return Err(format!(
                    "Couldn't download dependency ({:?}): {:?}",
                    dependency_name, e
                ))
            }
        };

        // Mutate this dependency's path to hold the newly downloaded dependency's path.
        details.path = Some(downloaded_dep_path);
    }
    let dep_path = match dependency_lib {
        Dependency::Simple(..) => {
            return Err(
                "Not yet implemented: Simple version-spec dependencies require a registry.".into(),
            )
        }
        Dependency::Detailed(DependencyDetails { path, .. }) => path,
    };

    let dep_path =
        match dep_path {
            Some(p) => p,
            None => return Err(
                "Only simple path imports are supported right now. Please supply a path relative \
                 to the manifest file."
                    .into(),
            ),
        };

    // dependency paths are relative to the path of the project being compiled
    let mut project_path = PathBuf::from(project_file_path);
    project_path.push(dep_path);

    // compile the dependencies of this dependency
    // this should detect circular dependencies
    let manifest_dir = match find_manifest_dir(&project_path) {
        Some(o) => o,
        None => {
            return Err(format!(
                "Manifest not found for dependency {:?}.",
                project_path
            ))
        }
    };
    let mut manifest_of_dep = read_manifest(&manifest_dir)?;
    let main_path = find_main_path(&manifest_dir, &manifest_of_dep);
    let file_name = find_file_name(&manifest_dir, &main_path)?;

    let build_config = BuildConfig::root_from_file_name_and_manifest_path(
        file_name.to_path_buf(),
        manifest_dir.clone(),
    );

    let dep_namespace = create_module();
    if let Some(ref mut deps) = manifest_of_dep.dependencies {
        for (dependency_name, ref mut dependency_lib) in deps {
            // to do this properly, iterate over list of dependencies make sure there are no
            // circular dependencies
            compile_dependency_lib(
                &manifest_dir,
                dependency_name,
                dependency_lib,
                dep_namespace,
                dependency_graph,
                silent_mode,
                offline_mode,
            )?;
        }
    }

    let main_file = get_main_file(&manifest_of_dep, &manifest_dir)?;

    let compiled = compile_library(
        main_file,
        &manifest_of_dep.project.name,
        dep_namespace,
        build_config,
        dependency_graph,
        silent_mode,
    )?;

    namespace.insert_module_ref(dependency_name.to_string(), compiled);

    // nothing is returned from this method since it mutates the hashmaps it was given
    Ok(())
}

fn compile_library(
    source: Arc<str>,
    proj_name: &str,
    namespace: NamespaceRef,
    build_config: BuildConfig,
    dependency_graph: &mut HashMap<String, HashSet<String>>,
    silent_mode: bool,
) -> Result<NamespaceRef, String> {
    let res = sway_core::compile_to_asm(source, namespace, build_config, dependency_graph);
    match res {
        CompilationResult::Library {
            namespace,
            warnings,
            ..
        } => {
            print_on_success_library(silent_mode, proj_name, warnings);
            Ok(namespace)
        }
        CompilationResult::Failure { errors, warnings } => {
            print_on_failure(silent_mode, warnings, errors);
            Err(format!("Failed to compile {}", proj_name))
        }
        _ => {
            return Err(format!(
                "Project \"{}\" was included as a dependency but it is not a library.",
                proj_name
            ))
        }
    }
}

fn compile(
    source: Arc<str>,
    proj_name: &str,
    namespace: NamespaceRef,
    build_config: BuildConfig,
    dependency_graph: &mut HashMap<String, HashSet<String>>,
    source_map: &mut SourceMap,
    silent_mode: bool,
) -> Result<Vec<u8>, String> {
    let res = sway_core::compile_to_bytecode(
        source,
        namespace,
        build_config,
        dependency_graph,
        source_map,
    );

    match res {
        BytecodeCompilationResult::Success { bytes, warnings } => {
            print_on_success(silent_mode, proj_name, warnings, TreeType::Script {});
            Ok(bytes)
        }
        BytecodeCompilationResult::Library { warnings } => {
            print_on_success_library(silent_mode, proj_name, warnings);
            Ok(vec![])
        }
        BytecodeCompilationResult::Failure { errors, warnings } => {
            print_on_failure(silent_mode, warnings, errors);
            Err(format!("Failed to compile {}", proj_name))
        }
    }
}

fn compile_to_asm(
    source: Arc<str>,
    proj_name: &str,
    namespace: NamespaceRef,
    build_config: BuildConfig,
    dependency_graph: &mut HashMap<String, HashSet<String>>,
    silent_mode: bool,
) -> Result<FinalizedAsm, String> {
    let res = sway_core::compile_to_asm(source, namespace, build_config, dependency_graph);
    match res {
        CompilationResult::Success { asm, warnings } => {
            print_on_success(silent_mode, proj_name, warnings, TreeType::Script {});
            Ok(asm)
        }
        CompilationResult::Library { warnings, .. } => {
            print_on_success_library(silent_mode, proj_name, warnings);
            Ok(FinalizedAsm::Library)
        }
        CompilationResult::Failure { errors, warnings } => {
            print_on_failure(silent_mode, warnings, errors);
            return Err(format!("Failed to compile {}", proj_name));
        }
    }
}
