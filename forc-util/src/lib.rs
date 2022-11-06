//! Utility items shared between forc crates.

use annotate_snippets::{
    display_list::{DisplayList, FormatOptions},
    snippet::{Annotation, AnnotationType, Slice, Snippet, SourceAnnotation},
};
use anyhow::{bail, Result};
use forc_fs::Filesystem;
use forc_tracing::{println_green_err, println_red_err, println_yellow_err};
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::str;
use sway_core::language::parsed::TreeType;
use sway_error::error::CompileError;
use sway_error::warning::CompileWarning;
use sway_types::{LineCol, Spanned};
use sway_utils::constants;

pub mod restricted;

pub const DEFAULT_OUTPUT_DIRECTORY: &str = "out";

/// Continually go up in the file tree until a specified file is found.
#[allow(clippy::branches_sharing_code)]
pub fn find_parent_dir_with_file(starter_path: &Path, file_name: &str) -> Option<PathBuf> {
    let mut path = std::fs::canonicalize(starter_path).ok()?;
    let empty_path = PathBuf::from("/");
    while path != empty_path {
        path.push(file_name);
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
/// Continually go up in the file tree until a Forc manifest file is found.
pub fn find_manifest_dir(starter_path: &Path) -> Option<PathBuf> {
    find_parent_dir_with_file(starter_path, constants::MANIFEST_FILE_NAME)
}

pub fn is_sway_file(file: &Path) -> bool {
    let res = file.extension();
    Some(OsStr::new(constants::SWAY_EXTENSION)) == res
}

pub fn find_file_name<'sc>(manifest_dir: &Path, entry_path: &'sc Path) -> Result<&'sc Path> {
    let mut file_path = manifest_dir.to_path_buf();
    file_path.pop();
    let file_name = match entry_path.strip_prefix(file_path.clone()) {
        Ok(o) => o,
        Err(err) => bail!(err),
    };
    Ok(file_name)
}

pub fn lock_path(manifest_dir: &Path) -> PathBuf {
    manifest_dir.join(constants::LOCK_FILE_NAME)
}

// Using (https://github.com/rust-lang/cargo/blob/489b66f2e458404a10d7824194d3ded94bc1f4e4/src/cargo/util/toml/mod.rs +
// https://github.com/rust-lang/cargo/blob/489b66f2e458404a10d7824194d3ded94bc1f4e4/src/cargo/ops/cargo_new.rs) for reference

pub fn validate_name(name: &str, use_case: &str) -> Result<()> {
    // if true returns formatted error
    restricted::contains_invalid_char(name, use_case)?;

    if restricted::is_keyword(name) {
        bail!("the name `{name}` cannot be used as a package name, it is a Sway keyword");
    }
    if restricted::is_conflicting_artifact_name(name) {
        bail!(
            "the name `{name}` cannot be used as a package name, \
            it conflicts with Forc's build directory names"
        );
    }
    if name.to_lowercase() == "test" {
        bail!(
            "the name `test` cannot be used as a project name, \
            it conflicts with Sway's built-in test library"
        );
    }
    if restricted::is_conflicting_suffix(name) {
        bail!(
            "the name `{name}` is part of Sway's standard library\n\
            It is recommended to use a different name to avoid problems."
        );
    }
    if restricted::is_windows_reserved(name) {
        if cfg!(windows) {
            bail!("cannot use name `{name}`, it is a reserved Windows filename");
        } else {
            bail!(
                "the name `{name}` is a reserved Windows filename\n\
                This package will not work on Windows platforms."
            );
        }
    }
    if restricted::is_non_ascii_name(name) {
        bail!("the name `{name}` contains non-ASCII characters which are unsupported");
    }
    Ok(())
}

/// Simple function to convert kebab-case to snake_case.
pub fn kebab_to_snake_case(s: &str) -> String {
    s.replace('-', "_")
}

pub fn default_output_directory(manifest_dir: &Path) -> PathBuf {
    manifest_dir.join(DEFAULT_OUTPUT_DIRECTORY)
}

/// Returns the user's `.forc` directory, `$HOME/.forc` by default.
pub fn user_forc_directory() -> Filesystem {
    Filesystem::new(
        dirs::home_dir()
            .expect("unable to find the user home directory")
            .join(constants::USER_FORC_DIRECTORY),
    )
}

/// The location at which `forc` will checkout git repositories.
pub fn git_checkouts_directory() -> Filesystem {
    user_forc_directory().join("git").join("checkouts")
}

pub fn print_on_success(
    terse_mode: bool,
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

    if !terse_mode {
        warnings.iter().for_each(format_warning);
    }

    if warnings.is_empty() {
        println_green_err(&format!("  Compiled {} {:?}.", type_str, proj_name));
    } else {
        println_yellow_err(&format!(
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

pub fn print_on_success_library(terse_mode: bool, proj_name: &str, warnings: &[CompileWarning]) {
    if !terse_mode {
        warnings.iter().for_each(format_warning);
    }

    if warnings.is_empty() {
        println_green_err(&format!("  Compiled library {:?}.", proj_name));
    } else {
        println_yellow_err(&format!(
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

pub fn print_on_failure(terse_mode: bool, warnings: &[CompileWarning], errors: &[CompileError]) {
    let e_len = errors.len();

    if !terse_mode {
        warnings.iter().for_each(format_warning);
        errors.iter().for_each(format_err);
    }

    println_red_err(&format!(
        "  Aborting due to {} {}.",
        e_len,
        if e_len > 1 { "errors" } else { "error" }
    ));
}

fn format_err(err: &CompileError) {
    let span = err.span();
    let input = span.input();
    let path = err.path();
    let path_str = path.as_ref().map(|path| path.to_string_lossy());
    let mut start_pos = span.start();
    let mut end_pos = span.end();

    let friendly_str = maybe_uwuify(&format!("{}", err));
    let (snippet_title, snippet_slices) = if start_pos < end_pos {
        let title = Some(Annotation {
            label: None,
            id: None,
            annotation_type: AnnotationType::Error,
        });

        let (mut start, end) = err.span().line_col();
        let input = construct_window(&mut start, end, &mut start_pos, &mut end_pos, input);
        let slices = vec![Slice {
            source: input,
            line_start: start.line,
            origin: path_str.as_deref(),
            fold: false,
            annotations: vec![SourceAnnotation {
                label: &friendly_str,
                annotation_type: AnnotationType::Error,
                range: (start_pos, end_pos),
            }],
        }];

        (title, slices)
    } else {
        (
            Some(Annotation {
                label: Some(friendly_str.as_str()),
                id: None,
                annotation_type: AnnotationType::Error,
            }),
            Vec::new(),
        )
    };

    let snippet = Snippet {
        title: snippet_title,
        footer: vec![],
        slices: snippet_slices,
        opt: FormatOptions {
            color: true,
            ..Default::default()
        },
    };
    tracing::error!("{}\n____\n", DisplayList::from(snippet))
}

fn format_warning(err: &CompileWarning) {
    let span = err.span();
    let input = span.input();
    let path = err.path();
    let path_str = path.as_ref().map(|path| path.to_string_lossy());

    let friendly_str = maybe_uwuify(&err.to_friendly_warning_string());
    let mut start_pos = span.start();
    let mut end_pos = span.end();
    if start_pos == end_pos {
        // if start/pos are same we will not get that arrow pointing to code, so we add +1.
        end_pos += 1;
    }

    let (mut start, end) = err.span.line_col();
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
            origin: path_str.as_deref(),
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
    tracing::warn!("{}\n____\n", DisplayList::from(snippet))
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
    debug_assert!(total_lines_in_input >= total_lines_of_highlight);

    let mut current_line = 0;
    let mut lines_to_start_of_snippet = 0;
    let mut calculated_start_ix = None;
    let mut calculated_end_ix = None;
    let mut pos = 0;
    for character in input.chars() {
        if character == '\n' {
            current_line += 1
        }

        if current_line + NUM_LINES_BUFFER >= start.line && calculated_start_ix.is_none() {
            calculated_start_ix = Some(pos);
            lines_to_start_of_snippet = current_line;
        }

        if current_line >= end.line + NUM_LINES_BUFFER && calculated_end_ix.is_none() {
            calculated_end_ix = Some(pos);
        }

        if calculated_start_ix.is_some() && calculated_end_ix.is_some() {
            break;
        }
        pos += character.len_utf8();
    }
    let calculated_start_ix = calculated_start_ix.unwrap_or(0);
    let calculated_end_ix = calculated_end_ix.unwrap_or(input.len());

    let start_ix_bytes = *start_ix - std::cmp::min(calculated_start_ix, *start_ix);
    let end_ix_bytes = *end_ix - std::cmp::min(calculated_start_ix, *end_ix);
    // We want the start_ix and end_ix in terms of chars and not bytes, so translate.
    *start_ix = input[calculated_start_ix..(calculated_start_ix + start_ix_bytes)]
        .chars()
        .count();
    *end_ix = input[calculated_start_ix..(calculated_start_ix + end_ix_bytes)]
        .chars()
        .count();

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
