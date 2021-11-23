use crate::utils::dependency::{Dependency, DependencyDetails};
use crate::{
    cli::JsonAbiCommand,
    utils::dependency,
    utils::helpers::{
        find_file_name, find_main_path, find_manifest_dir, get_main_file, print_on_failure,
        print_on_success_library, print_on_success_script, read_manifest,
    },
};

use core_types::Function;

use anyhow::Result;
use core_lang::{BuildConfig, CompilationResult, LibraryExports, Namespace};
use serde_json::{json, Value};
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::path::PathBuf;

pub fn build(command: JsonAbiCommand) -> Result<Value, String> {
    // find manifest directory, even if in subdirectory
    let this_dir = if let Some(ref path) = command.path {
        PathBuf::from(path)
    } else {
        std::env::current_dir().map_err(|e| format!("{:?}", e))?
    };

    let JsonAbiCommand {
        json_outfile,
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
    let main_path = find_main_path(&manifest_dir, &manifest);
    let file_name = find_file_name(&manifest_dir, &main_path)?;

    let build_config = BuildConfig::root_from_file_name_and_manifest_path(
        file_name.to_owned(),
        manifest_dir.clone(),
    );

    let mut dependency_graph = HashMap::new();
    let mut json_abi = vec![];

    let mut namespace: Namespace = Default::default();
    if let Some(ref mut deps) = manifest.dependencies {
        for (dependency_name, dependency_details) in deps.iter_mut() {
            // Check if dependency is a git-based dependency.
            let dep = match dependency_details {
                Dependency::Simple(..) => {
                    return Err(
                        "Not yet implemented: Simple version-spec dependencies require a registry."
                            .into(),
                    );
                }
                Dependency::Detailed(dep_details) => dep_details,
            };

            // Download a non-local dependency if the `git` property is set in this dependency.
            if let Some(git) = &dep.git {
                let downloaded_dep_path = match dependency::download_github_dep(
                    dependency_name,
                    git,
                    &dep.branch,
                    &dep.version,
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
                dep.path = Some(downloaded_dep_path);
            }

            json_abi.append(&mut compile_dependency_lib(
                &this_dir,
                dependency_name,
                dependency_details,
                &mut namespace,
                &mut dependency_graph,
                silent_mode,
            )?);
        }
    }

    // now, compile this program with all of its dependencies
    let main_file = get_main_file(&manifest, &manifest_dir)?;

    json_abi.append(&mut compile(
        main_file,
        &manifest.project.name,
        &namespace,
        build_config,
        &mut dependency_graph,
        silent_mode,
    )?);

    let output_json = json!(json_abi);

    if let Some(outfile) = json_outfile {
        let file = File::create(outfile).map_err(|e| e.to_string())?;
        serde_json::to_writer(&file, &output_json.clone()).map_err(|e| e.to_string())?;
    } else {
        println!("{}", output_json);
    }

    Ok(output_json)
}

/// Takes a dependency and returns a namespace of exported things from that dependency
/// trait implementations are included as well
fn compile_dependency_lib<'source, 'manifest>(
    project_file_path: &PathBuf,
    dependency_name: &'manifest str,
    dependency_lib: &Dependency,
    namespace: &mut Namespace<'source>,
    dependency_graph: &mut HashMap<String, HashSet<String>>,
    silent_mode: bool,
) -> Result<Vec<Function>, String> {
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
    let mut project_path = project_file_path.clone();
    project_path.push(dep_path);

    // compile the dependencies of this dependency
    // this should detect circular dependencies
    let manifest_dir = match find_manifest_dir(&project_path) {
        Some(o) => o,
        None => return Err("Manifest not found for dependency.".into()),
    };
    let manifest_of_dep = read_manifest(&manifest_dir)?;
    let main_path = find_main_path(&manifest_dir, &manifest_of_dep);
    let file_name = find_file_name(&manifest_dir, &main_path)?;

    let build_config = BuildConfig::root_from_file_name_and_manifest_path(
        file_name.to_owned(),
        manifest_dir.clone(),
    );
    let mut dep_namespace = namespace.clone();

    // The part below here is just a massive shortcut to get the standard library working
    if let Some(ref deps) = manifest_of_dep.dependencies {
        for dep in deps {
            // to do this properly, iterate over list of dependencies make sure there are no
            // circular dependencies
            //return Err("Unimplemented: dependencies that have dependencies".into());
            compile_dependency_lib(
                project_file_path,
                dep.0,
                dep.1,
                // give it a cloned namespace, which we then merge with this namespace
                &mut dep_namespace,
                dependency_graph,
                silent_mode,
            )?;
        }
    }

    let main_file = get_main_file(&manifest_of_dep, &manifest_dir)?;

    let (compiled, json_abi) = compile_library(
        main_file,
        &manifest_of_dep.project.name,
        &dep_namespace,
        build_config.clone(),
        dependency_graph,
        silent_mode,
    )?;

    namespace.insert_dependency_module(dependency_name.to_string(), compiled.namespace);

    // nothing is returned from this method since it mutates the hashmaps it was given
    Ok(json_abi)
}

fn compile_library<'source, 'manifest>(
    source: &'source str,
    proj_name: &str,
    namespace: &Namespace<'source>,
    build_config: BuildConfig,
    dependency_graph: &mut HashMap<String, HashSet<String>>,
    silent_mode: bool,
) -> Result<(LibraryExports<'source>, Vec<Function>), String> {
    let res = core_lang::compile_to_asm(&source, namespace, build_config, dependency_graph);
    match res {
        CompilationResult::Library {
            warnings,
            json_abi,
            exports,
        } => {
            print_on_success_library(silent_mode, proj_name, warnings);
            Ok((exports, json_abi))
        }
        CompilationResult::Success { warnings, .. } => {
            print_on_failure(silent_mode, warnings, vec![]);
            Err(format!("Failed to compile {}", proj_name))
        }
        CompilationResult::Failure { warnings, errors } => {
            print_on_failure(silent_mode, warnings, errors);
            Err(format!("Failed to compile {}", proj_name))
        }
    }
}

fn compile<'source, 'manifest>(
    source: &'source str,
    proj_name: &str,
    namespace: &Namespace<'source>,
    build_config: BuildConfig,
    dependency_graph: &mut HashMap<String, HashSet<String>>,
    silent_mode: bool,
) -> Result<Vec<Function>, String> {
    let res = core_lang::compile_to_asm(&source, namespace, build_config, dependency_graph);
    match res {
        CompilationResult::Success {
            warnings, json_abi, ..
        } => {
            print_on_success_script(silent_mode, proj_name, warnings);
            Ok(json_abi)
        }
        CompilationResult::Library { warnings, .. } => {
            print_on_failure(silent_mode, warnings, vec![]);
            Err(format!("Failed to compile {}", proj_name))
        }
        CompilationResult::Failure { warnings, errors } => {
            print_on_failure(silent_mode, warnings, errors);
            Err(format!("Failed to compile {}", proj_name))
        }
    }
}
