use crate::utils::dependency::{Dependency, DependencyDetails};
use crate::utils::helpers::{find_file_name, find_main_path};
use crate::{
    cli::BuildCommand,
    utils::dependency,
    utils::helpers::{
        default_output_directory, get_main_file, print_on_failure, print_on_success,
        print_on_success_library, read_manifest,
    },
};
use std::fs::{self, File};
use std::io::Write;
use std::sync::Arc;
use sway_core::{FinalizedAsm, TreeType};
use sway_utils::{find_manifest_dir, MANIFEST_FILE_NAME};

use sway_core::{
    create_module, source_map::SourceMap, BuildConfig, BytecodeCompilationResult,
    CompilationResult, CompileAstResult, NamespaceRef, NamespaceWrapper, TypedParseTree,
};
use sway_types::JsonABI;

use anyhow::Result;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

pub fn build(command: BuildCommand) -> Result<(Vec<u8>, JsonABI), String> {
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

    // find manifest directory, even if in subdirectory
    let this_dir = if let Some(ref path) = path {
        PathBuf::from(path)
    } else {
        std::env::current_dir().map_err(|e| format!("{:?}", e))?
    };

    let manifest_dir = match find_manifest_dir(&this_dir) {
        Some(dir) => dir,
        None => {
            return Err(format!(
                "could not find `{}` in `{}` or any parent directory",
                MANIFEST_FILE_NAME,
                this_dir.display(),
            ))
        }
    };

    let mut manifest = read_manifest(&manifest_dir)?;
    let main_path = find_main_path(&manifest_dir, &manifest);
    let file_name = find_file_name(&manifest_dir, &main_path)?;

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

    let mut source_map = SourceMap::new();
    let mut json_abi = vec![];

    if let Some(ref mut deps) = manifest.dependencies {
        for (dependency_name, dependency_details) in deps.iter_mut() {
            let dep_json_abi = compile_dependency_lib(
                &this_dir,
                dependency_name,
                dependency_details,
                namespace,
                &mut dependency_graph,
                silent_mode,
                offline_mode,
            )?;
            json_abi.extend(dep_json_abi);

            source_map.insert_dependency(match dependency_details {
                Dependency::Simple(..) => {
                    todo!("simple deps (compile_dependency_lib should have errored on this)");
                }
                Dependency::Detailed(DependencyDetails { path, .. }) => path
                    .as_ref()
                    .expect("compile_dependency_lib should have set this")
                    .clone(),
            });
        }
    }

    // now, compile this program with all of its dependencies
    let main_file = get_main_file(&manifest, &manifest_dir)?;

    let (main, main_json_abi) = compile(
        main_file,
        &manifest.project.name,
        namespace,
        build_config,
        &mut dependency_graph,
        &mut source_map,
        silent_mode,
    )?;

    json_abi.extend(main_json_abi);

    if let Some(outfile) = binary_outfile {
        let mut file = File::create(outfile).map_err(|e| e.to_string())?;
        file.write_all(main.as_slice()).map_err(|e| e.to_string())?;
    }

    if let Some(outfile) = debug_outfile {
        fs::write(
            outfile,
            &serde_json::to_vec(&source_map).expect("JSON serialization failed"),
        )
        .map_err(|e| e.to_string())?;
    }

    // TODO: We may support custom build profiles in the future.
    let profile = "debug";

    // Create the output directory for build artifacts.
    let output_dir = output_directory
        .map(PathBuf::from)
        .unwrap_or_else(|| default_output_directory(&manifest_dir).join(profile));
    if !output_dir.exists() {
        fs::create_dir_all(&output_dir).map_err(|e| e.to_string())?;
    }

    // Place build artifacts into the output directory.
    let bin_path = output_dir
        .join(&manifest.project.name)
        .with_extension("bin");
    std::fs::write(&bin_path, main.as_slice()).map_err(|e| e.to_string())?;
    if !json_abi.is_empty() {
        let json_abi_stem = format!("{}-abi", manifest.project.name);
        let json_abi_path = output_dir.join(&json_abi_stem).with_extension("json");
        let file = File::create(json_abi_path).map_err(|e| e.to_string())?;
        let res = if minify_json_abi {
            serde_json::to_writer(&file, &json_abi)
        } else {
            serde_json::to_writer_pretty(&file, &json_abi)
        };
        res.map_err(|e| e.to_string())?;
    }

    println!("  Bytecode size is {} bytes.", main.len());

    Ok((main, json_abi))
}

/// Takes a dependency and returns a namespace of exported things from that dependency including
/// trait implementations.
///
/// Also returns the JSON ABI of the library. This may be empty in the case that no `abi` was
/// exposed.
fn compile_dependency_lib<'manifest>(
    project_file_path: &Path,
    dependency_name: &'manifest str,
    dependency_lib: &mut Dependency,
    namespace: NamespaceRef,
    dependency_graph: &mut HashMap<String, HashSet<String>>,
    silent_mode: bool,
    offline_mode: bool,
) -> Result<JsonABI, String> {
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

    let (compiled, json_abi) = compile_library(
        main_file,
        &manifest_of_dep.project.name,
        dep_namespace,
        build_config,
        dependency_graph,
        silent_mode,
    )?;

    namespace.insert_module_ref(dependency_name.to_string(), compiled);

    Ok(json_abi)
}

fn compile_library(
    source: Arc<str>,
    proj_name: &str,
    namespace: NamespaceRef,
    build_config: BuildConfig,
    dependency_graph: &mut HashMap<String, HashSet<String>>,
    silent_mode: bool,
) -> Result<(NamespaceRef, JsonABI), String> {
    let res = sway_core::compile_to_ast(source, namespace, &build_config, dependency_graph);
    match res {
        CompileAstResult::Success {
            parse_tree,
            tree_type,
            warnings,
        } => {
            let errors = vec![];
            match tree_type {
                TreeType::Library { .. } => {
                    print_on_success_library(silent_mode, proj_name, &warnings);
                    let json_abi = generate_json_abi(&*parse_tree);
                    let namespace = parse_tree.get_namespace_ref();
                    Ok((namespace, json_abi))
                }
                _ => {
                    print_on_failure(silent_mode, &warnings, &errors);
                    Err(format!("Failed to compile {}", proj_name))
                }
            }
        }
        CompileAstResult::Failure { warnings, errors } => {
            print_on_failure(silent_mode, &warnings, &errors);
            Err(format!("Failed to compile {}", proj_name))
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
) -> Result<(Vec<u8>, JsonABI), String> {
    let ast_res = sway_core::compile_to_ast(source, namespace, &build_config, dependency_graph);
    let json_abi = match &ast_res {
        CompileAstResult::Success {
            parse_tree,
            tree_type,
            warnings,
        } => match tree_type {
            TreeType::Library { .. } => {
                print_on_failure(silent_mode, warnings, &[]);
                return Err(format!("Failed to compile {}", proj_name));
            }
            typ => {
                print_on_success(silent_mode, proj_name, warnings, typ.clone());
                generate_json_abi(&*parse_tree)
            }
        },
        CompileAstResult::Failure { warnings, errors } => {
            print_on_failure(silent_mode, warnings, errors);
            return Err(format!("Failed to compile {}", proj_name));
        }
    };

    let asm_res = sway_core::ast_to_asm(ast_res, &build_config);
    let bc_res = sway_core::asm_to_bytecode(asm_res, source_map);

    let bytes = match bc_res {
        BytecodeCompilationResult::Success { bytes, warnings } => {
            print_on_success(silent_mode, proj_name, &warnings, TreeType::Script {});
            bytes
        }
        BytecodeCompilationResult::Library { warnings } => {
            print_on_success_library(silent_mode, proj_name, &warnings);
            vec![]
        }
        BytecodeCompilationResult::Failure { errors, warnings } => {
            print_on_failure(silent_mode, &warnings, &errors);
            return Err(format!("Failed to compile {}", proj_name));
        }
    };
    Ok((bytes, json_abi))
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
            print_on_success(silent_mode, proj_name, &warnings, TreeType::Script {});
            Ok(asm)
        }
        CompilationResult::Library { warnings, .. } => {
            print_on_success_library(silent_mode, proj_name, &warnings);
            Ok(FinalizedAsm::Library)
        }
        CompilationResult::Failure { errors, warnings } => {
            print_on_failure(silent_mode, &warnings, &errors);
            return Err(format!("Failed to compile {}", proj_name));
        }
    }
}

fn generate_json_abi(ast: &TypedParseTree) -> JsonABI {
    match ast {
        TypedParseTree::Contract { abi_entries, .. } => {
            abi_entries.iter().map(|x| x.generate_json_abi()).collect()
        }
        _ => vec![],
    }
}
