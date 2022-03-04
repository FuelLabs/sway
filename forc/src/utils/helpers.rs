use super::manifest::Manifest;
use crate::utils::restricted_names;
use annotate_snippets::{
    display_list::{DisplayList, FormatOptions},
    snippet::{Annotation, AnnotationType, Slice, Snippet, SourceAnnotation},
};
use anyhow::{anyhow, bail, Result};
use std::ffi::OsStr;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::str;
use std::sync::Arc;
use sway_core::{error::LineCol, CompileError, CompileWarning, TreeType};
use sway_utils::constants;
use termcolor::{self, Color as TermColor, ColorChoice, ColorSpec, StandardStream, WriteColor};

pub const DEFAULT_OUTPUT_DIRECTORY: &str = "out";

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

pub fn find_file_name<'sc>(manifest_dir: &Path, main_path: &'sc Path) -> Result<&'sc Path> {
    let mut file_path = manifest_dir.to_path_buf();
    file_path.pop();
    let file_name = match main_path.strip_prefix(file_path.clone()) {
        Ok(o) => o,
        Err(err) => bail!(err),
    };
    Ok(file_name)
}

pub fn lock_path(manifest_dir: &Path) -> PathBuf {
    manifest_dir.join(constants::LOCK_FILE_NAME)
}

pub fn read_manifest(manifest_dir: &Path) -> Result<Manifest> {
    let manifest_path = {
        let mut man = PathBuf::from(manifest_dir);
        man.push(constants::MANIFEST_FILE_NAME);
        man
    };
    let manifest_path_str = format!("{:?}", manifest_path);
    let manifest = match std::fs::read_to_string(manifest_path) {
        Ok(o) => o,
        Err(e) => {
            bail!("failed to read manifest at {:?}: {}", manifest_path_str, e)
        }
    };

    let manifest = match toml::from_str(&manifest) {
        Ok(o) => Ok(o),
        Err(e) => Err(anyhow!("Error parsing manifest: {}.", e)),
    }?;

    validate_manifest(manifest)
}

// Using (https://github.com/rust-lang/cargo/blob/489b66f2e458404a10d7824194d3ded94bc1f4e4/src/cargo/util/toml/mod.rs +
// https://github.com/rust-lang/cargo/blob/489b66f2e458404a10d7824194d3ded94bc1f4e4/src/cargo/ops/cargo_new.rs) for reference

fn validate_name(name: &str, use_case: &str) -> Result<()> {
    // if true returns formatted error
    restricted_names::contains_invalid_char(name, use_case)?;

    if restricted_names::is_keyword(name) {
        bail!("the name `{name}` cannot be used as a package name, it is a Sway keyword");
    }
    if restricted_names::is_conflicting_artifact_name(name) {
        bail!(
            "the name `{name}` cannot be used as a package name, \
            it conflicts with Forc's build directory names"
        );
    }
    if name == "test" {
        bail!(
            "the name `test` cannot be used as a package name, \
            it conflicts with Sway's built-in test library"
        );
    }
    if restricted_names::is_conflicting_suffix(name) {
        bail!(
            "the name `{name}` is part of Sway's standard library\n\
            It is recommended to use a different name to avoid problems."
        );
    }
    if restricted_names::is_windows_reserved(name) {
        if cfg!(windows) {
            bail!("cannot use name `{name}`, it is a reserved Windows filename");
        } else {
            bail!(
                "the name `{name}` is a reserved Windows filename\n\
                This package will not work on Windows platforms."
            );
        }
    }
    if restricted_names::is_non_ascii_name(name) {
        bail!("the name `{name}` contains non-ASCII characters which are unsupported");
    }
    Ok(())
}

fn validate_manifest(manifest: Manifest) -> Result<Manifest> {
    validate_name(&manifest.project.name, "package name")?;
    if let Some(ref org) = manifest.project.organization {
        validate_name(org, "organization name")?;
    }

    Ok(manifest)
}

pub fn get_main_file(manifest_of_dep: &Manifest, manifest_dir: &Path) -> Result<Arc<str>> {
    let main_path = {
        let mut code_dir = PathBuf::from(manifest_dir);
        code_dir.push(constants::SRC_DIR);
        code_dir.push(&manifest_of_dep.project.entry);
        code_dir
    };

    // some hackery to get around lifetimes for now, until the AST returns a non-lifetime-bound AST
    let main_file = std::fs::read_to_string(&main_path).map_err(|e| e)?;
    let main_file = Arc::from(main_file);
    Ok(main_file)
}

pub fn default_output_directory(manifest_dir: &Path) -> PathBuf {
    manifest_dir.join(DEFAULT_OUTPUT_DIRECTORY)
}

/// Returns the user's `.forc` directory, `$HOME/.forc` by default.
pub fn user_forc_directory() -> PathBuf {
    dirs::home_dir()
        .expect("unable to find the user home directory")
        .join(constants::USER_FORC_DIRECTORY)
}

/// The location at which `forc` will checkout git repositories.
pub fn git_checkouts_directory() -> PathBuf {
    user_forc_directory().join("git").join("checkouts")
}

pub fn print_on_success(
    silent_mode: bool,
    proj_name: &str,
    warnings: &[CompileWarning],
    tree_type: &TreeType,
) {
    let type_str = match &tree_type {
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

pub fn print_on_success_library(silent_mode: bool, proj_name: &str, warnings: &[CompileWarning]) {
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

pub fn print_on_failure(silent_mode: bool, warnings: &[CompileWarning], errors: &[CompileError]) {
    let e_len = errors.len();

    if !silent_mode {
        warnings.iter().for_each(format_warning);
        errors.iter().for_each(format_err);
    }

    println_red_err(&format!(
        "  Aborting due to {} {}.",
        e_len,
        if e_len > 1 { "errors" } else { "error" }
    ))
    .unwrap();
}

pub(crate) fn print_lock_diff(proj_name: &str, diff: &crate::lock::Diff) {
    print_removed_pkgs(proj_name, diff.removed.iter().cloned());
    print_added_pkgs(proj_name, diff.added.iter().cloned());
}

pub(crate) fn print_removed_pkgs<'a, I>(proj_name: &str, removed: I)
where
    I: IntoIterator<Item = &'a crate::lock::PkgLock>,
{
    for pkg in removed {
        if pkg.name != proj_name {
            let _ = println_red(&format!("  Removing {}", pkg.unique_string()));
        }
    }
}

pub(crate) fn print_added_pkgs<'a, I>(proj_name: &str, removed: I)
where
    I: IntoIterator<Item = &'a crate::lock::PkgLock>,
{
    for pkg in removed {
        if pkg.name != proj_name {
            let _ = println_green(&format!("    Adding {}", pkg.unique_string()));
        }
    }
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

    let (mut start_pos, mut end_pos) = err.span();
    if start_pos == end_pos {
        // if start/pos are same we will not get that arrow pointing to code, so we add +1.
        end_pos += 1;
    }
    let friendly_str = maybe_uwuify(&err.to_friendly_error_string());
    let (mut start, end) = err.line_col();
    let input = construct_window(&mut start, end, &mut start_pos, &mut end_pos, input);
    let snippet = Snippet {
        title: Some(Annotation {
            label: None,
            id: None,
            annotation_type: AnnotationType::Error,
        }),
        footer: vec![],
        slices: vec![Slice {
            source: input,
            line_start: start.line,
            origin: Some(&path),
            fold: false,
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

    let friendly_str = maybe_uwuify(&err.to_friendly_warning_string());
    let (mut start_pos, mut end_pos) = err.span();
    if start_pos == end_pos {
        // if start/pos are same we will not get that arrow pointing to code, so we add +1.
        end_pos += 1;
    }

    let (mut start, end) = err.line_col();
    let input = construct_window(&mut start, end, &mut start_pos, &mut end_pos, input);
    let snippet = Snippet {
        title: Some(Annotation {
            label: None,
            id: None,
            annotation_type: AnnotationType::Warning,
        }),
        footer: vec![],
        slices: vec![Slice {
            source: input,
            line_start: start.line,
            origin: Some(&path),
            fold: false,
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

/// Given a start and an end position and an input, determine how much of a window to show in the
/// error.
/// Mutates the start and end indexes to be in line with the new slice length.
///
/// The library we use doesn't handle auto-windowing and line numbers, so we must manually
/// calculate the line numbers and match them up with the input window. It is a bit fiddly.t
fn construct_window<'a>(
    start: &mut LineCol,
    end: LineCol,
    start_ix: &mut usize,
    end_ix: &mut usize,
    input: &'a str,
) -> &'a str {
    // how many lines to prepend or append to the highlighted region in the window
    const NUM_LINES_BUFFER: usize = 2;

    let total_lines_in_input = input.chars().filter(|x| *x == '\n').count();
    debug_assert!(end.line >= start.line);
    let total_lines_of_highlight = end.line - start.line;
    debug_assert!(total_lines_in_input > total_lines_of_highlight);

    let mut current_line = 0;
    let mut lines_to_start_of_snippet = 0;
    let mut calculated_start_ix = None;
    let mut calculated_end_ix = None;
    for (ix, character) in input.chars().enumerate() {
        if character == '\n' {
            current_line += 1
        }

        if current_line + NUM_LINES_BUFFER >= start.line && calculated_start_ix.is_none() {
            calculated_start_ix = Some(ix);
            lines_to_start_of_snippet = current_line;
        }

        if current_line >= end.line + NUM_LINES_BUFFER && calculated_end_ix.is_none() {
            calculated_end_ix = Some(ix);
        }

        if calculated_start_ix.is_some() && calculated_end_ix.is_some() {
            break;
        }
    }
    let calculated_start_ix = calculated_start_ix.unwrap_or(0);
    let calculated_end_ix = calculated_end_ix.unwrap_or(input.len());

    *start_ix -= std::cmp::min(calculated_start_ix, *start_ix);
    *end_ix -= std::cmp::min(calculated_start_ix, *end_ix);
    start.line = lines_to_start_of_snippet;
    &input[calculated_start_ix..calculated_end_ix]
}

#[cfg(all(feature = "uwu", any(target_arch = "x86", target_arch = "x86_64")))]
fn maybe_uwuify(raw: &str) -> String {
    use uwuifier::uwuify_str_sse;
    uwuify_str_sse(raw)
}
#[cfg(all(feature = "uwu", not(any(target_arch = "x86", target_arch = "x86_64"))))]
fn maybe_uwuify(raw: &str) -> String {
    compile_error!("The `uwu` feature only works on x86 or x86_64 processors.");
    Default::default()
}

#[cfg(not(feature = "uwu"))]
fn maybe_uwuify(raw: &str) -> String {
    raw.to_string()
}
