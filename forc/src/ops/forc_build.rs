use crate::utils::dependency::{Dependency, DependencyDetails};
use crate::{
    cli::BuildCommand,
    utils::dependency,
    utils::helpers::{
        find_manifest_dir, get_main_file, print_blue_err, println_green_err, println_red_err,
        println_yellow_err, read_manifest,
    },
};
use line_col::LineColLookup;
use source_span::{
    fmt::{Color, Formatter, Style},
    Position, Span,
};
use std::fs::File;
use std::io::Write;

use anyhow::Result;
use core_lang::{
    BuildConfig, BytecodeCompilationResult, CompilationResult, FinalizedAsm, LibraryExports,
    Namespace,
};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

pub fn build(command: BuildCommand) -> Result<Vec<u8>, String> {
    // find manifest directory, even if in subdirectory
    let this_dir = if let Some(ref path) = command.path {
        PathBuf::from(path)
    } else {
        std::env::current_dir().unwrap()
    };

    let BuildCommand {
        path,
        binary_outfile,
        print_finalized_asm,
        print_intermediate_asm,
        offline_mode,
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
        code_dir.push(crate::utils::constants::SRC_DIR);
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
        file_name.clone().to_path_buf(),
        manifest_dir.clone(),
    )
    .print_finalized_asm(print_finalized_asm)
    .print_intermediate_asm(print_intermediate_asm);

    let mut dependency_graph = HashMap::new();

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

            compile_dependency_lib(
                &this_dir,
                &dependency_name,
                &dependency_details,
                &mut namespace,
                &mut dependency_graph,
            )?;
        }
    }

    // now, compile this program with all of its dependencies
    let main_file = get_main_file(&manifest, &manifest_dir)?;

    let main = compile(
        main_file,
        &manifest.project.name,
        &namespace,
        build_config,
        &mut dependency_graph,
    )?;
    if let Some(outfile) = binary_outfile {
        let mut file = File::create(outfile).map_err(|e| e.to_string())?;
        file.write_all(main.as_slice()).map_err(|e| e.to_string())?;
    }

    println!("Bytecode size is {} bytes.", main.len());

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

    let main_path = {
        let mut code_dir = manifest_dir.clone();
        code_dir.push(crate::utils::constants::SRC_DIR);
        code_dir.push(&manifest_of_dep.project.entry);
        code_dir
    };
    let mut file_path = manifest_dir.clone();
    file_path.pop();
    let file_name = match main_path.strip_prefix(file_path.clone()) {
        Ok(o) => o,
        Err(err) => return Err(err.to_string()),
    };

    let build_config = BuildConfig::root_from_file_name_and_manifest_path(
        file_name.clone().to_path_buf(),
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
                &dep.0,
                &dep.1,
                // give it a cloned namespace, which we then merge with this namespace
                &mut dep_namespace,
                dependency_graph,
            )?;
        }
    }

    let main_file = get_main_file(&manifest_of_dep, &manifest_dir)?;

    let compiled = compile_library(
        main_file,
        &manifest_of_dep.project.name,
        &dep_namespace,
        build_config.clone(),
        dependency_graph,
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
) -> Result<LibraryExports<'source>, String> {
    let res = core_lang::compile_to_asm(&source, namespace, build_config, dependency_graph);
    match res {
        CompilationResult::Library { exports, warnings } => {
            warnings.iter().for_each(|warning| format_warning(warning));

            if warnings.is_empty() {
                let _ = println_green_err(&format!("Compiled library {:?}.", proj_name));
            } else {
                let _ = println_yellow_err(&format!(
                    "Compiled library {:?} with {} {}.",
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

            warnings.iter().for_each(|warning| format_warning(warning));

            errors.into_iter().for_each(|error| format_err(error));

            println_red_err(&format!(
                "Aborting due to {} {}.",
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
) -> Result<Vec<u8>, String> {
    let res = core_lang::compile_to_bytecode(&source, namespace, build_config, dependency_graph);
    match res {
        BytecodeCompilationResult::Success { bytes, warnings } => {
            warnings.iter().for_each(|warning| format_warning(warning));

            if warnings.is_empty() {
                let _ = println_green_err(&format!("Compiled script {:?}.", proj_name));
            } else {
                let _ = println_yellow_err(&format!(
                    "Compiled script {:?} with {} {}.",
                    proj_name,
                    warnings.len(),
                    if warnings.len() > 1 {
                        "warnings"
                    } else {
                        "warning"
                    }
                ));
            }
            Ok(bytes)
        }
        BytecodeCompilationResult::Library { warnings } => {
            warnings.iter().for_each(|warning| format_warning(warning));

            if warnings.is_empty() {
                let _ = println_green_err(&format!("Compiled library {:?}.", proj_name));
            } else {
                let _ = println_yellow_err(&format!(
                    "Compiled library {:?} with {} {}.",
                    proj_name,
                    warnings.len(),
                    if warnings.len() > 1 {
                        "warnings"
                    } else {
                        "warning"
                    }
                ));
            }
            Ok(vec![])
        }
        BytecodeCompilationResult::Failure { errors, warnings } => {
            let e_len = errors.len();

            warnings.iter().for_each(|warning| format_warning(warning));

            errors.into_iter().for_each(|error| format_err(error));

            println_red_err(&format!(
                "Aborting due to {} {}.",
                e_len,
                if e_len > 1 { "errors" } else { "error" }
            ))
            .unwrap();
            Err(format!("Failed to compile {}", proj_name))
        }
    }
}

fn format_warning(err: &core_lang::CompileWarning) {
    let input = err.span.input();
    let chars = input.chars().map(|x| -> Result<_, ()> { Ok(x) });

    let metrics = source_span::DEFAULT_METRICS;
    let buffer = source_span::SourceBuffer::new(chars, Position::default(), metrics);

    let mut fmt = Formatter::with_margin_color(Color::Blue);

    for c in buffer.iter() {
        let _ = c.unwrap(); // report eventual errors.
    }

    let (start_pos, end_pos) = err.span();
    let lookup = LineColLookup::new(input);
    let (start_line, start_col) = lookup.get(start_pos);
    let (end_line, end_col) = lookup.get(end_pos - 1);

    let err_start = Position::new(start_line - 1, start_col - 1);
    let err_end = Position::new(end_line - 1, end_col - 1);
    let err_span = Span::new(err_start, err_end, err_end.next_column());
    fmt.add(
        err_span,
        Some(err.to_friendly_warning_string()),
        Style::Warning,
    );

    let formatted = fmt.render(buffer.iter(), buffer.span(), &metrics).unwrap();

    print_blue_err(" --> ").unwrap();
    print!("{}", err.path());
    println!("{}", formatted);
}

fn format_err(err: core_lang::CompileError) {
    let input = err.internal_span().input();
    let chars = input.chars().map(|x| -> Result<_, ()> { Ok(x) });

    let metrics = source_span::DEFAULT_METRICS;
    let buffer = source_span::SourceBuffer::new(chars, Position::default(), metrics);

    let mut fmt = Formatter::with_margin_color(Color::Blue);

    for c in buffer.iter() {
        let _ = c.unwrap(); // report eventual errors.
    }

    let (start_pos, end_pos) = err.span();
    let lookup = LineColLookup::new(input);
    let (start_line, start_col) = lookup.get(start_pos);
    let (end_line, end_col) = lookup.get(if end_pos == 0 { 0 } else { end_pos - 1 });

    let err_start = Position::new(start_line - 1, start_col - 1);
    let err_end = Position::new(end_line - 1, end_col - 1);
    let err_span = Span::new(err_start, err_end, err_end.next_column());
    fmt.add(err_span, Some(err.to_friendly_error_string()), Style::Error);

    let formatted = fmt.render(buffer.iter(), buffer.span(), &metrics).unwrap();
    fmt.add(
        buffer.span(),
        Some("this is the whole program\nwhat a nice program!".to_string()),
        Style::Error,
    );

    print_blue_err(" --> ").unwrap();
    print!("{}", err.path());
    println!("{}", formatted);
}

fn compile_to_asm<'source, 'manifest>(
    source: &'source str,
    proj_name: &str,
    namespace: &Namespace<'source>,
    build_config: BuildConfig,
    dependency_graph: &mut HashMap<String, HashSet<String>>,
) -> Result<FinalizedAsm<'source>, String> {
    let res = core_lang::compile_to_asm(&source, namespace, build_config, dependency_graph);
    match res {
        CompilationResult::Success { asm, warnings } => {
            warnings.iter().for_each(|warning| format_warning(warning));

            if warnings.is_empty() {
                let _ = println_green_err(&format!("Compiled script {:?}.", proj_name));
            } else {
                let _ = println_yellow_err(&format!(
                    "Compiled script {:?} with {} {}.",
                    proj_name,
                    warnings.len(),
                    if warnings.len() > 1 {
                        "warnings"
                    } else {
                        "warning"
                    }
                ));
            }
            Ok(asm)
        }
        CompilationResult::Library { warnings, .. } => {
            warnings.iter().for_each(|warning| format_warning(warning));

            if warnings.is_empty() {
                let _ = println_green_err(&format!("Compiled library {:?}.", proj_name));
            } else {
                let _ = println_yellow_err(&format!(
                    "Compiled library {:?} with {} {}.",
                    proj_name,
                    warnings.len(),
                    if warnings.len() > 1 {
                        "warnings"
                    } else {
                        "warning"
                    }
                ));
            }
            Ok(FinalizedAsm::Library)
        }
        CompilationResult::Failure { errors, warnings } => {
            let e_len = errors.len();

            warnings.iter().for_each(|warning| format_warning(warning));

            errors.into_iter().for_each(|error| format_err(error));

            println_red_err(&format!(
                "Aborting due to {} {}.",
                e_len,
                if e_len > 1 { "errors" } else { "error" }
            ))
            .unwrap();
            Err(format!("Failed to compile {}", proj_name))
        }
    }
}
