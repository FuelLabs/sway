use crate::utils::dependency::{Dependency, DependencyDetails};
use crate::{
    cli::AbiSpecCommand,
    utils::dependency,
    utils::helpers::{find_manifest_dir, get_main_file, read_manifest},
};

use core_lang::Namespace;

use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

pub fn generate_abi_spec(command: AbiSpecCommand) -> Result<Vec<u8>, String> {
    let AbiSpecCommand {
        path,
        offline_mode,
        silent_mode,
        json_outfile,
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

    // create dependency graph and load dependencies
    let mut dependency_graph = HashMap::new();
    let mut namespace: Namespace = Default::default();
    if let Some(ref mut deps) = manifest.dependencies {
        for (dependency_name, dependency_details) in deps.iter_mut() {
            dependency::resolve_dependency(
                dependency_name.clone(),
                dependency_details,
                offline_mode,
            )?;
            generate_abi_spec_lib(
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
    let main = generate_abi_spec_main(
        main_file,
        &manifest.project.name,
        &namespace,
        &mut dependency_graph,
        silent_mode,
    )?;
    if let Some(outfile) = json_outfile {
        let mut file = File::create(outfile).map_err(|e| e.to_string())?;
        file.write_all(main.as_slice()).map_err(|e| e.to_string())?;
    } else {
        println!("{:#?}", main);
    }

    Ok(main)
}

fn generate_abi_spec_lib<'source, 'manifest>(
    project_file_path: &PathBuf,
    dependency_name: &'manifest str,
    dependency_lib: &Dependency,
    namespace: &mut Namespace<'source>,
    dependency_graph: &mut HashMap<String, HashSet<String>>,
    silent_mode: bool,
) -> Result<(), String> {
    // find the dependency paths
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
    let mut project_path = project_file_path.clone();
    project_path.push(dep_path);

    // resolve the manifest
    let manifest_dir = match find_manifest_dir(&project_path) {
        Some(o) => o,
        None => return Err("Manifest not found for dependency.".into()),
    };
    let manifest_of_dep = read_manifest(&manifest_dir)?;

    // compile the dependencies of this dependency
    let mut dep_namespace = namespace.clone();
    if let Some(ref deps) = manifest_of_dep.dependencies {
        for dep in deps {
            generate_abi_spec_lib(
                project_file_path,
                &dep.0,
                &dep.1,
                &mut dep_namespace,
                dependency_graph,
                silent_mode,
            )?;
        }
    }

    let main_file = get_main_file(&manifest_of_dep, &manifest_dir)?;

    let compiled = generate_abi_spec_library(
        main_file,
        &manifest_of_dep.project.name,
        &dep_namespace,
        dependency_graph,
        silent_mode,
    )?;

    namespace.insert_dependency_module(dependency_name.to_string(), /*compiled.namespace*/ todo!());

    // nothing is returned from this method since it mutates the hashmaps it was given
    //Ok(())

    unimplemented!()
}

fn generate_abi_spec_main<'source, 'manifest>(
    source: &'source str,
    proj_name: &str,
    namespace: &Namespace<'source>,
    dependency_graph: &mut HashMap<String, HashSet<String>>,
    silent_mode: bool,
) -> Result<Vec<u8>, String> {
    unimplemented!()
}

fn generate_abi_spec_library<'source, 'manifest>(
    source: &'source str,
    proj_name: &str,
    namespace: &Namespace<'source>,
    dependency_graph: &mut HashMap<String, HashSet<String>>,
    silent_mode: bool,
) -> Result<Vec<u8>, String> {
    let res = core_lang::abi_spec::generate_abi_spec();

    unimplemented!()
}