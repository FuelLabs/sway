use crate::utils::dependency::{Dependency, DependencyDetails};
use crate::{
    cli::BuildCommand,
    utils::dependency,
    utils::helpers::{
        find_manifest_dir, format_err, format_warning, get_file_name, get_main_file, get_main_path,
        print_blue_err, println_green_err, println_red_err, println_yellow_err, read_manifest,
    },
};
use std::fs::File;
use std::io::Write;

use anyhow::Result;
use core_lang::{
    BuildConfig, BytecodeCompilationResult, CompilationResult, LibraryExports, Namespace,
};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

pub fn build(command: BuildCommand) -> Result<Vec<u8>, String> {
    let BuildCommand {
        binary_outfile,
        print_finalized_asm,
        print_intermediate_asm,
        offline_mode,
        silent_mode,
        path,
        ..
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
                "No manifest file found in this directory or any parent directories of it: {:?}",
                this_dir
            ))
        }
    };
    let mut manifest = read_manifest(&manifest_dir)?;
    let main_path = get_main_path(&manifest, &manifest_dir);
    let file_name = get_file_name(&manifest_dir, &main_path)?;

    let mut dependency_graph = HashMap::new();

    let mut namespace: Namespace = Default::default();
    if let Some(ref mut deps) = manifest.dependencies {
        for (dependency_name, dependency_details) in deps.iter_mut() {
            dependency::resolve_dependency(
                dependency_name.clone(),
                dependency_details,
                offline_mode,
            )?;
            compile_dependency_lib(
                &this_dir,
                &dependency_name,
                &dependency_details,
                &mut namespace,
                &mut dependency_graph,
                silent_mode,
            )?;
        }
    }

    // now, compile this program with all of its dependencies
    let main_file = get_main_file(&manifest, &manifest_dir)?;

    let build_config = BuildConfig::root_from_file_name_and_manifest_path(
        file_name.clone().to_path_buf(),
        manifest_dir.clone(),
    )
    .print_finalized_asm(print_finalized_asm)
    .print_intermediate_asm(print_intermediate_asm);

    let main = compile(
        main_file,
        &manifest.project.name,
        &namespace,
        build_config,
        &mut dependency_graph,
        silent_mode,
    )?;

    if let Some(outfile) = binary_outfile {
        let mut file = File::create(outfile).map_err(|e| e.to_string())?;
        file.write_all(main.as_slice()).map_err(|e| e.to_string())?;
    }

    println!("  Bytecode size is {} bytes.", main.len());

    Ok(main)
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
) -> Result<(), String> {
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
    let main_path = get_main_path(&manifest_of_dep, &manifest_dir);
    let file_name = get_file_name(&manifest_dir, &main_path)?;

    let mut dep_namespace = namespace.clone();

    // The part below here is just a massive shortcut to get the standard library working
    if let Some(ref deps) = manifest_of_dep.dependencies {
        for dep in deps {
            // to do this properly, iterate over list of dependencies make sure there are no
            // circular dependencies
            //return Err("Unimplemented: dependencies that have dependencies".into());
            compile_dependency_lib(
                project_file_path,
                &dep.0,
                &dep.1,
                // give it a cloned namespace, which we then merge with this namespace
                &mut dep_namespace,
                dependency_graph,
                silent_mode,
            )?;
        }
    }

    let main_file = get_main_file(&manifest_of_dep, &manifest_dir)?;

    let build_config = BuildConfig::root_from_file_name_and_manifest_path(
        file_name.clone().to_path_buf(),
        manifest_dir.clone(),
    );

    let compiled = compile_library(
        main_file,
        &manifest_of_dep.project.name,
        &dep_namespace,
        build_config.clone(),
        dependency_graph,
        silent_mode,
    )?;

    namespace.insert_dependency_module(dependency_name.to_string(), compiled.namespace);

    // nothing is returned from this method since it mutates the hashmaps it was given
    Ok(())
}

fn compile_library<'source, 'manifest>(
    source: &'source str,
    proj_name: &str,
    namespace: &Namespace<'source>,
    build_config: BuildConfig,
    dependency_graph: &mut HashMap<String, HashSet<String>>,
    silent_mode: bool,
) -> Result<LibraryExports<'source>, String> {
    let res = core_lang::compile_to_asm(&source, namespace, build_config, dependency_graph);
    match res {
        CompilationResult::Library { exports, warnings } => {
            if !silent_mode {
                warnings.iter().for_each(|warning| format_warning(warning));
            }

            if warnings.is_empty() {
                let _ = println_green_err(&format!("  Compiled library {:?}.", proj_name));
            } else {
                let _ = println_yellow_err(&format!(
                    "  Compiled library {:?} with {} {}.",
                    proj_name,
                    warnings.len(),
                    if warnings.len() > 1 {
                        "warnings"
                    } else {
                        "warning"
                    }
                ));
            }
            Ok(exports)
        }
        CompilationResult::Failure { errors, warnings } => {
            let e_len = errors.len();

            if !silent_mode {
                warnings.iter().for_each(|warning| format_warning(warning));
                errors.into_iter().for_each(|error| format_err(&error));
            }

            println_red_err(&format!(
                "  Aborting due to {} {}.",
                e_len,
                if e_len > 1 { "errors" } else { "error" }
            ))
            .unwrap();
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

fn compile<'source, 'manifest>(
    source: &'source str,
    proj_name: &str,
    namespace: &Namespace<'source>,
    build_config: BuildConfig,
    dependency_graph: &mut HashMap<String, HashSet<String>>,
    silent_mode: bool,
) -> Result<Vec<u8>, String> {
    let res = core_lang::compile_to_bytecode(&source, namespace, build_config, dependency_graph);
    match res {
        BytecodeCompilationResult::Success { bytes, warnings } => {
            if !silent_mode {
                warnings.iter().for_each(|warning| format_warning(warning));
            }

            if warnings.is_empty() {
                let _ = println_green_err(&format!("  Compiled script {:?}.", proj_name));
            } else {
                let _ = println_yellow_err(&format!(
                    "  Compiled script {:?} with {} {}.",
                    proj_name,
                    warnings.len(),
                    if warnings.len() > 1 {
                        "warnings"
                    } else {
                        "warning"
                    }
                ));
            }
            return Ok(bytes);
        }
        BytecodeCompilationResult::Library { warnings } => {
            if !silent_mode {
                warnings.iter().for_each(|warning| format_warning(warning));
            }

            if warnings.is_empty() {
                let _ = println_green_err(&format!("  Compiled library {:?}.", proj_name));
            } else {
                let _ = println_yellow_err(&format!(
                    "  Compiled library {:?} with {} {}.",
                    proj_name,
                    warnings.len(),
                    if warnings.len() > 1 {
                        "warnings"
                    } else {
                        "warning"
                    }
                ));
            }
            return Ok(vec![]);
        }
        BytecodeCompilationResult::Failure { errors, warnings } => {
            let e_len = errors.len();

            if !silent_mode {
                warnings.iter().for_each(|warning| format_warning(warning));
                errors.into_iter().for_each(|error| format_err(&error));
            }

            println_red_err(&format!(
                "  Aborting due to {} {}.",
                e_len,
                if e_len > 1 { "errors" } else { "error" }
            ))
            .unwrap();
            return Err(format!("Failed to compile {}", proj_name));
        }
    }
}
