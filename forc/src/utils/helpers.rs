use super::manifest::Manifest;
use annotate_snippets::{
    display_list::{DisplayList, FormatOptions},
    snippet::{Annotation, AnnotationType, Slice, Snippet, SourceAnnotation},
};
use std::ffi::OsStr;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::str;
use sway_core::{CompileError, CompileWarning, TreeType};
use sway_utils::constants;
use termcolor::{self, Color as TermColor, ColorChoice, ColorSpec, StandardStream, WriteColor};

pub fn is_sway_file(file: &Path) -> bool {
    let res = file.extension();
    Some(OsStr::new(constants::SWAY_EXTENSION)) == res
}

pub fn find_main_path(manifest_dir: &Path, manifest: &Manifest) -> PathBuf {
    let mut code_dir = manifest_dir.to_path_buf();
    code_dir.push(constants::SRC_DIR);
    code_dir.push(&manifest.project.entry);
    code_dir
}

pub fn find_file_name<'sc>(manifest_dir: &Path, main_path: &'sc Path) -> Result<&'sc Path, String> {
    let mut file_path = manifest_dir.to_path_buf();
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
        man.push(constants::MANIFEST_FILE_NAME);
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
        code_dir.push(constants::SRC_DIR);
        code_dir.push(&manifest_of_dep.project.entry);
        code_dir
    };

    // some hackery to get around lifetimes for now, until the AST returns a non-lifetime-bound AST
    let main_file = std::fs::read_to_string(&main_path).map_err(|e| e.to_string())?;
    let main_file = Box::new(main_file);
    let main_file: &'static mut String = Box::leak(main_file);
    Ok(main_file)
}

pub fn print_on_success<'sc>(
    silent_mode: bool,
    proj_name: &str,
    warnings: Vec<CompileWarning>,
    tree_type: TreeType<'sc>,
) {
    let type_str = match tree_type {
        TreeType::Script {} => "script",
        TreeType::Contract {} => "contract",
        TreeType::Predicate {} => "predicate",
        TreeType::Library { .. } => "library",
    };

    if !silent_mode {
        warnings.iter().for_each(format_warning);
    }

    if warnings.is_empty() {
        let _ = println_green_err(&format!("  Compiled {} {:?}.", type_str, proj_name));
    } else {
        let _ = println_yellow_err(&format!(
            "  Compiled {} {:?} with {} {}.",
            type_str,
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
        warnings.iter().for_each(format_warning);
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
}

pub fn print_on_failure(
    silent_mode: bool,
    warnings: Vec<CompileWarning>,
    errors: Vec<CompileError>,
) {
    let e_len = errors.len();

    if !silent_mode {
        warnings.iter().for_each(format_warning);
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

fn format_err(err: &sway_core::CompileError) {
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

fn format_warning(err: &sway_core::CompileWarning) {
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
