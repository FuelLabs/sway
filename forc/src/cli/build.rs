use line_col::LineColLookup;
use source_span::{
    fmt::{Color, Formatter, Style},
    Position, Span,
};
use std::fs::File;
use std::io::{self, Write};
use termcolor::{BufferWriter, Color as TermColor, ColorChoice, ColorSpec, WriteColor};

use crate::manifest::{Dependency, DependencyDetails, Manifest};
use std::{fs, path::PathBuf};

pub(crate) fn build() -> Result<(), String> {
    // find manifest directory, even if in subdirectory
    let this_dir = std::env::current_dir().unwrap();
    dbg!(&this_dir);
    let manifest_dir = match find_manifest_dir(&this_dir) {
        Some(dir) => dir,
        None => {
            return Err(format!(
                "No manifest file found in this directory or any parent directories of it: {:?}",
                this_dir
            ))
        }
    };

    Ok(())
}

/// Continually go up in the file tree until a manifest (Fuel.toml) is found.
fn find_manifest_dir(starter_path: &PathBuf) -> Option<PathBuf> {
    let mut path = starter_path.clone();
    let empty_path = PathBuf::from("/");
    while path != empty_path {
        path.push(crate::constants::MANIFEST_FILE_NAME);
        println!("Checking {:?}", path);
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
fn compile_dependency_lib(dependency_lib: Dependency) -> Result<(), String> {
    let dep_path = match dependency_lib {
        Dependency::Simple(..) => {
            return Err("Simple version-spec dependencies require a registry.".into())
        }
        Dependency::Detailed(DependencyDetails { path, .. }) => path,
    };

    let dep_path = match dep_path {
        Some(p) => p,
        None => return Err("Only simple path imports are supported right now. Please supply a path relative to the manifest file.".into())
    };

    // compile the dependencies of this dependency
    //this should detect circular dependencies
    let manifest_dir = match find_manifest_dir(&PathBuf::from(dep_path)) {
        Some(o) => o,
        None => return Err("Manifest not found for dependency.".into()),
    };

    let manifest_of_dep = read_manifest(&manifest_dir)?;

    // The part below here is just a massive shortcut to get the standard library working
    if let Some(deps) = manifest_of_dep.dependencies {
        if deps.len() > 0 {
            return Err("Unimplemented: dependencies that have dependencies".into());
        }
    }

    // another shortcut -- ignoring manifest and compiling main file directly
    let main_path = {
        let mut code_dir = manifest_dir.clone();
        code_dir.push("/src/main.fm");
        code_dir
    };
    let compiled = compile(main_path);

    todo!("What to return here? a list of compiled functions?")
    // i think this is where functions should be copied into the syntax tree with concrete
    // types
}

fn read_manifest(manifest_dir: &PathBuf) -> Result<Manifest, String> {
    let manifest_path = {
        let mut man = manifest_dir.clone();
        man.push(crate::constants::MANIFEST_FILE_NAME);
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

fn compile(path: PathBuf) -> Result<LibraryExports, String> {
    let main_file = fs::read_to_string(&path).map_err(|e| e.to_string())?;
    let res = parser::compile(&main_file);
    match res {
        Ok((compiled, warnings)) => {
            for ref warning in warnings.iter() {
                format_warning(&main_file, warning);
            }
            if warnings.is_empty() {
                let _ = write_green(&format!("Successfully compiled {:?}.", path));
            } else {
                let _ = write_yellow(&format!(
                    "Compiled {:?} with {} {}.",
                    path,
                    warnings.len(),
                    if warnings.len() > 1 {
                        "warnings"
                    } else {
                        "warning"
                    }
                ));
            }
        }
        Err((errors, warnings)) => {
            let e_len = errors.len();

            for ref warning in warnings.iter() {
                format_warning(&main_file, warning);
            }

            errors.into_iter().for_each(|e| format_err(&main_file, e));

            write_red(format!(
                "Aborting due to {} {}.",
                e_len,
                if e_len > 1 { "errors" } else { "error" }
            ))
            .unwrap();
        }
    }
    todo!()
}

fn format_warning(input: &str, err: &parser::CompileWarning) {
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
    fmt.add(
        buffer.span(),
        Some("this is the whole program\nwhat a nice program!".to_string()),
        Style::Error,
    );

    println!("{}", formatted);
}

fn format_err(input: &str, err: parser::CompileError) {
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
