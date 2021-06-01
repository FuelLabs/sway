use line_col::LineColLookup;
use source_span::{
    fmt::{Color, Formatter, Style},
    Position, Span,
};
use std::io::{self, Write};
use termcolor::{BufferWriter, Color as TermColor, ColorChoice, ColorSpec, WriteColor};

use crate::utils::manifest::{Dependency, DependencyDetails, Manifest};
use core_lang::{
    BytecodeCompilationResult, CompilationResult, FinalizedAsm, LibraryExports, Namespace,
};
use std::{fs, path::PathBuf};

pub fn print_asm(path: Option<String>) -> Result<(), String> {
    // find manifest directory, even if in subdirectory
    let this_dir = if let Some(path) = path {
        PathBuf::from(path)
    } else {
        std::env::current_dir().unwrap()
    };
    let manifest_dir = find_manifest_dir(&this_dir).ok_or(format!(
        "No manifest file found in this directory or any parent directories of it: {:?}",
        this_dir
    ))?;
    let manifest = read_manifest(&manifest_dir)?;

    let mut namespace: Namespace = Default::default();
    if let Some(ref deps) = manifest.dependencies {
        for (dependency_name, dependency_details) in deps.iter() {
            compile_dependency_lib(
                &this_dir,
                &dependency_name,
                &dependency_details,
                &mut namespace,
            )?;
        }
    }

    // now, compile this program with all of its dependencies
    let main_file = get_main_file(&manifest, &manifest_dir)?;
    let main = compile_to_asm(main_file, &manifest.project.name, &namespace)?;

    println!("{}", main);

    Ok(())
}

pub fn build(path: Option<String>) -> Result<Vec<u8>, String> {
    // find manifest directory, even if in subdirectory
    let this_dir = if let Some(path) = path {
        PathBuf::from(path)
    } else {
        std::env::current_dir().unwrap()
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
    let manifest = read_manifest(&manifest_dir)?;

    let mut namespace: Namespace = Default::default();
    if let Some(ref deps) = manifest.dependencies {
        for (dependency_name, dependency_details) in deps.iter() {
            compile_dependency_lib(
                &this_dir,
                &dependency_name,
                &dependency_details,
                &mut namespace,
            )?;
        }
    }

    // now, compile this program with all of its dependencies
    let main_file = get_main_file(&manifest, &manifest_dir)?;
    let main = compile(main_file, &manifest.project.name, &namespace)?;

    println!("Bytecode size is {} bytes.", main.len());

    Ok(main)
}

/// Continually go up in the file tree until a manifest (Forc.toml) is found.
fn find_manifest_dir(starter_path: &PathBuf) -> Option<PathBuf> {
    let mut path = fs::canonicalize(starter_path.clone()).ok()?;
    let empty_path = PathBuf::from("/");
    while path != empty_path {
        path.push(crate::utils::constants::MANIFEST_FILE_NAME);
        if path.exists() {
            path.pop();
            return Some(path);
        } else {
            path.pop();
            path.pop();
        }
    }
    None
}

/// Takes a dependency and returns a namespace of exported things from that dependency
/// trait implementations are included as well
fn compile_dependency_lib<'source, 'manifest>(
    project_path: &PathBuf,
    dependency_name: &'manifest str,
    dependency_lib: &Dependency,
    namespace: &mut Namespace<'source>,
) -> Result<(), String> {
    let dep_path = match dependency_lib {
        Dependency::Simple(..) => {
            return Err("Simple version-spec dependencies require a registry.".into())
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
    let mut project_path = project_path.clone();
    project_path.push(dep_path);

    // compile the dependencies of this dependency
    // this should detect circular dependencies
    let manifest_dir = match find_manifest_dir(&project_path) {
        Some(o) => o,
        None => return Err("Manifest not found for dependency.".into()),
    };

    let manifest_of_dep = read_manifest(&manifest_dir)?;

    // The part below here is just a massive shortcut to get the standard library working
    if let Some(ref deps) = manifest_of_dep.dependencies {
        if deps.len() > 0 {
            // to do this properly, iterate over list of dependencies make sure there are no
            // circular dependencies
            return Err("Unimplemented: dependencies that have dependencies".into());
        }
    }

    let main_file = get_main_file(&manifest_of_dep, &manifest_dir)?;

    let compiled = compile_library(main_file, &manifest_of_dep.project.name, &namespace.clone())?;

    namespace.insert_module(dependency_name.to_string(), compiled.namespace);

    // nothing is returned from this method since it mutates the hashmaps it was given
    Ok(())
}

fn read_manifest(manifest_dir: &PathBuf) -> Result<Manifest, String> {
    let manifest_path = {
        let mut man = manifest_dir.clone();
        man.push(crate::utils::constants::MANIFEST_FILE_NAME);
        man
    };
    let manifest_path_str = format!("{:?}", manifest_path);
    let manifest = match std::fs::read_to_string(manifest_path) {
        Ok(o) => o,
        Err(e) => {
            return Err(format!(
                "failed to read manifest at {:?}: {}",
                manifest_path_str, e
            ))
        }
    };
    match toml::from_str(&manifest) {
        Ok(o) => Ok(o),
        Err(e) => Err(format!("Error parsing manifest: {}.", e)),
    }
}

fn compile_library<'source, 'manifest>(
    source: &'source str,
    proj_name: &str,
    namespace: &Namespace<'source>,
) -> Result<LibraryExports<'source>, String> {
    let res = core_lang::compile_to_asm(&source, namespace);
    match res {
        CompilationResult::Library { exports, warnings } => {
            for ref warning in warnings.iter() {
                format_warning(&source, warning);
            }
            if warnings.is_empty() {
                let _ = write_green(&format!("Compiled library {:?}.", proj_name));
            } else {
                let _ = write_yellow(&format!(
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

            for ref warning in warnings.iter() {
                format_warning(&source, warning);
            }

            errors.into_iter().for_each(|e| format_err(&source, e));

            write_red(format!(
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
) -> Result<Vec<u8>, String> {
    let res = core_lang::compile_to_bytecode(&source, namespace);
    match res {
        BytecodeCompilationResult::Success { bytes, warnings } => {
            for ref warning in warnings.iter() {
                format_warning(&source, warning);
            }
            if warnings.is_empty() {
                let _ = write_green(&format!("Compiled script {:?}.", proj_name));
            } else {
                let _ = write_yellow(&format!(
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
            for ref warning in warnings.iter() {
                format_warning(&source, warning);
            }
            if warnings.is_empty() {
                let _ = write_green(&format!("Compiled library {:?}.", proj_name));
            } else {
                let _ = write_yellow(&format!(
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

            for ref warning in warnings.iter() {
                format_warning(&source, warning);
            }

            errors.into_iter().for_each(|e| format_err(&source, e));

            write_red(format!(
                "Aborting due to {} {}.",
                e_len,
                if e_len > 1 { "errors" } else { "error" }
            ))
            .unwrap();
            Err(format!("Failed to compile {}", proj_name))
        }
    }
}

fn format_warning(input: &str, err: &core_lang::CompileWarning) {
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

    println!("{}", formatted);
}

fn format_err(input: &str, err: core_lang::CompileError) {
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

    println!("{}", formatted);
}

fn write_red(txt: String) -> io::Result<()> {
    let txt = txt.as_str();
    let bufwtr = BufferWriter::stderr(ColorChoice::Always);
    let mut buffer = bufwtr.buffer();
    buffer.set_color(ColorSpec::new().set_fg(Some(TermColor::Red)))?;
    writeln!(&mut buffer, "{}", txt)?;
    bufwtr.print(&buffer)
}

fn write_green(txt: &str) -> io::Result<()> {
    let bufwtr = BufferWriter::stderr(ColorChoice::Always);
    let mut buffer = bufwtr.buffer();
    buffer.set_color(ColorSpec::new().set_fg(Some(TermColor::Green)))?;
    writeln!(&mut buffer, "{}", txt)?;
    bufwtr.print(&buffer)
}

fn write_yellow(txt: &str) -> io::Result<()> {
    let bufwtr = BufferWriter::stderr(ColorChoice::Always);
    let mut buffer = bufwtr.buffer();
    buffer.set_color(ColorSpec::new().set_fg(Some(TermColor::Yellow)))?;
    writeln!(&mut buffer, "{}", txt)?;
    bufwtr.print(&buffer)
}

fn get_main_file(
    manifest_of_dep: &Manifest,
    manifest_dir: &PathBuf,
) -> Result<&'static mut String, String> {
    let main_path = {
        let mut code_dir = manifest_dir.clone();
        code_dir.push("src");
        code_dir.push(&manifest_of_dep.project.entry);
        code_dir
    };

    // some hackery to get around lifetimes for now, until the AST returns a non-lifetime-bound AST
    let main_file = fs::read_to_string(&main_path).map_err(|e| e.to_string())?;
    let main_file = Box::new(main_file);
    let main_file: &'static mut String = Box::leak(main_file);
    return Ok(main_file);
}
fn compile_to_asm<'source, 'manifest>(
    source: &'source str,
    proj_name: &str,
    namespace: &Namespace<'source>,
) -> Result<FinalizedAsm<'source>, String> {
    let res = core_lang::compile_to_asm(&source, namespace);
    match res {
        CompilationResult::Success { asm, warnings } => {
            for ref warning in warnings.iter() {
                format_warning(&source, warning);
            }
            if warnings.is_empty() {
                let _ = write_green(&format!("Compiled script {:?}.", proj_name));
            } else {
                let _ = write_yellow(&format!(
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
            for ref warning in warnings.iter() {
                format_warning(&source, warning);
            }
            if warnings.is_empty() {
                let _ = write_green(&format!("Compiled library {:?}.", proj_name));
            } else {
                let _ = write_yellow(&format!(
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

            for ref warning in warnings.iter() {
                format_warning(&source, warning);
            }

            errors.into_iter().for_each(|e| format_err(&source, e));

            write_red(format!(
                "Aborting due to {} {}.",
                e_len,
                if e_len > 1 { "errors" } else { "error" }
            ))
            .unwrap();
            Err(format!("Failed to compile {}", proj_name))
        }
    }
}
