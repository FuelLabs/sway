use super::constants::{SRC_DIR, SWAY_EXTENSION};
use super::manifest::Manifest;
use annotate_snippets::{
    display_list::{DisplayList, FormatOptions},
    snippet::{Annotation, AnnotationType, Slice, Snippet, SourceAnnotation},
};
use core_lang::{CompileError, CompileWarning};
use std::ffi::OsStr;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::{fs, str};
use termcolor::{self, Color as TermColor, ColorChoice, ColorSpec, StandardStream, WriteColor};

pub fn is_sway_file(file: &Path) -> bool {
    let res = file.extension();
    Some(OsStr::new(SWAY_EXTENSION)) == res
}

pub fn get_sway_files(path: PathBuf) -> Vec<PathBuf> {
    let mut files = vec![];
    let mut dir_entries = vec![path];

    while let Some(next_dir) = dir_entries.pop() {
        if let Ok(read_dir) = fs::read_dir(next_dir) {
            for entry in read_dir.filter_map(|res| res.ok()) {
                let path = entry.path();

                if path.is_dir() {
                    dir_entries.push(path);
                } else if is_sway_file(&path) {
                    files.push(path)
                }
            }
        }
    }

    files
}

// Continually go up in the file tree until a manifest (Forc.toml) is found.
pub fn find_manifest_dir(starter_path: &Path) -> Option<PathBuf> {
    let mut path = std::fs::canonicalize(starter_path).ok()?;
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

pub fn find_main_path(manifest_dir: &PathBuf, manifest: &Manifest) -> PathBuf {
    let mut code_dir = manifest_dir.clone();
    code_dir.push(crate::utils::constants::SRC_DIR);
    code_dir.push(&manifest.project.entry);
    code_dir
}

pub fn find_file_name<'sc>(
    manifest_dir: &'sc PathBuf,
    main_path: &'sc PathBuf,
) -> Result<&'sc Path, String> {
    let mut file_path = manifest_dir.clone();
    file_path.pop();
    let file_name = match main_path.strip_prefix(file_path.clone()) {
        Ok(o) => o,
        Err(err) => return Err(err.to_string()),
    };
    Ok(file_name)
}

pub fn read_manifest(manifest_dir: &Path) -> Result<Manifest, String> {
    let manifest_path = {
        let mut man = PathBuf::from(manifest_dir);
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

pub fn get_main_file(
    manifest_of_dep: &Manifest,
    manifest_dir: &Path,
) -> Result<&'static mut String, String> {
    let main_path = {
        let mut code_dir = PathBuf::from(manifest_dir);
        code_dir.push(SRC_DIR);
        code_dir.push(&manifest_of_dep.project.entry);
        code_dir
    };

    // some hackery to get around lifetimes for now, until the AST returns a non-lifetime-bound AST
    let main_file = std::fs::read_to_string(&main_path).map_err(|e| e.to_string())?;
    let main_file = Box::new(main_file);
    let main_file: &'static mut String = Box::leak(main_file);
    Ok(main_file)
}

pub fn print_on_success_script(silent_mode: bool, proj_name: &str, warnings: Vec<CompileWarning>) {
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
}

pub fn print_on_success_library(silent_mode: bool, proj_name: &str, warnings: Vec<CompileWarning>) {
    if !silent_mode {
        warnings.iter().for_each(|warning| format_warning(warning));
    }

    if warnings.is_empty() {
        let _ = println_green_err(&format!("  Compiled script {:?}.", proj_name));
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
}

pub fn print_on_failure(
    silent_mode: bool,
    warnings: Vec<CompileWarning>,
    errors: Vec<CompileError>,
) {
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
}

pub fn println_red(txt: &str) -> io::Result<()> {
    println_std_out(txt, TermColor::Red)
}

pub fn println_green(txt: &str) -> io::Result<()> {
    println_std_out(txt, TermColor::Green)
}

pub fn print_blue_err(txt: &str) -> io::Result<()> {
    print_std_err(txt, TermColor::Blue)
}

pub fn println_yellow_err(txt: &str) -> io::Result<()> {
    println_std_err(txt, TermColor::Yellow)
}

pub fn println_red_err(txt: &str) -> io::Result<()> {
    println_std_err(txt, TermColor::Red)
}

pub fn println_green_err(txt: &str) -> io::Result<()> {
    println_std_err(txt, TermColor::Green)
}

fn print_std_out(txt: &str, color: TermColor) -> io::Result<()> {
    let stdout = StandardStream::stdout(ColorChoice::Always);
    print_with_color(txt, color, stdout)
}

fn println_std_out(txt: &str, color: TermColor) -> io::Result<()> {
    let stdout = StandardStream::stdout(ColorChoice::Always);
    println_with_color(txt, color, stdout)
}

fn print_std_err(txt: &str, color: TermColor) -> io::Result<()> {
    let stdout = StandardStream::stderr(ColorChoice::Always);
    print_with_color(txt, color, stdout)
}

fn println_std_err(txt: &str, color: TermColor) -> io::Result<()> {
    let stdout = StandardStream::stderr(ColorChoice::Always);
    println_with_color(txt, color, stdout)
}

fn print_with_color(txt: &str, color: TermColor, stream: StandardStream) -> io::Result<()> {
    let mut stream = stream;
    stream.set_color(ColorSpec::new().set_fg(Some(color)))?;
    write!(&mut stream, "{}", txt)?;
    stream.reset()?;
    Ok(())
}

fn println_with_color(txt: &str, color: TermColor, stream: StandardStream) -> io::Result<()> {
    let mut stream = stream;
    stream.set_color(ColorSpec::new().set_fg(Some(color)))?;
    writeln!(&mut stream, "{}", txt)?;
    stream.reset()?;
    Ok(())
}

fn format_err(err: &core_lang::CompileError) {
    let input = err.internal_span().input();
    let path = err.path();

    let (start_pos, mut end_pos) = err.span();
    if start_pos == end_pos {
        // if start/pos are same we will not get that arrow pointing to code, so we add +1.
        end_pos += 1;
    }
    let friendly_str = err.to_friendly_error_string();
    let snippet = Snippet {
        title: Some(Annotation {
            label: None,
            id: None,
            annotation_type: AnnotationType::Error,
        }),
        footer: vec![],
        slices: vec![Slice {
            source: input,
            line_start: 0,
            origin: Some(&path),
            fold: true,
            annotations: vec![SourceAnnotation {
                label: &friendly_str,
                annotation_type: AnnotationType::Error,
                range: (start_pos, end_pos),
            }],
        }],
        opt: FormatOptions {
            color: true,
            ..Default::default()
        },
    };
    eprintln!("{}", DisplayList::from(snippet))
}

fn format_warning(err: &core_lang::CompileWarning) {
    let input = err.span.input();
    let path = err.path();

    let (start_pos, mut end_pos) = err.span();
    let friendly_str = err.to_friendly_warning_string();
    if start_pos == end_pos {
        // if start/pos are same we will not get that arrow pointing to code, so we add +1.
        end_pos += 1;
    }
    let snippet = Snippet {
        title: Some(Annotation {
            label: None,
            id: None,
            annotation_type: AnnotationType::Warning,
        }),
        footer: vec![],
        slices: vec![Slice {
            source: input,
            line_start: 0,
            origin: Some(&path),
            fold: true,
            annotations: vec![SourceAnnotation {
                label: &friendly_str,
                annotation_type: AnnotationType::Warning,
                range: (start_pos, end_pos),
            }],
        }],
        opt: FormatOptions {
            color: true,
            ..Default::default()
        },
    };
    eprintln!("{}", DisplayList::from(snippet))
}
