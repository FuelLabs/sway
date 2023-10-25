//! Utility items shared between forc crates.

use annotate_snippets::{
    display_list::{DisplayList, FormatOptions},
    snippet::{Annotation, AnnotationType, Slice, Snippet, SourceAnnotation},
};
use ansi_term::Colour;
use anyhow::{bail, Context, Result};
use forc_tracing::{println_red_err, println_yellow_err};
use std::{
    collections::{hash_map, HashSet},
    fmt::Display,
    fs::File,
    hash::{Hash, Hasher},
    path::{Path, PathBuf},
    process::Termination,
    str,
};
use sway_core::language::parsed::TreeType;
use sway_error::{
    diagnostic::{Diagnostic, Issue, Label, LabelType, Level, ToDiagnostic},
    error::CompileError,
    warning::CompileWarning,
};
use sway_types::{LineCol, SourceEngine, Span};
use sway_utils::constants;
use tracing::error;

pub mod restricted;

pub const DEFAULT_OUTPUT_DIRECTORY: &str = "out";
pub const DEFAULT_ERROR_EXIT_CODE: u8 = 1;
pub const DEFAULT_SUCCESS_EXIT_CODE: u8 = 0;

/// A result type for forc operations. This shouldn't be returned from entry points, instead return
/// `ForcCliResult` to exit with correct exit code.
pub type ForcResult<T, E = ForcError> = Result<T, E>;

/// A wrapper around `ForcResult`. Designed to be returned from entry points as it handles
/// error reporting and exits with correct exit code.
#[derive(Debug)]
pub struct ForcCliResult<T> {
    result: ForcResult<T>,
}

/// A forc error type which is a wrapper around `anyhow::Error`. It enables propagation of custom
/// exit code alongisde the original error.
#[derive(Debug)]
pub struct ForcError {
    error: anyhow::Error,
    exit_code: u8,
}

impl ForcError {
    pub fn new(error: anyhow::Error, exit_code: u8) -> Self {
        Self { error, exit_code }
    }

    /// Returns a `ForcError` with provided exit_code.
    pub fn exit_code(self, exit_code: u8) -> Self {
        Self {
            error: self.error,
            exit_code,
        }
    }
}

impl AsRef<anyhow::Error> for ForcError {
    fn as_ref(&self) -> &anyhow::Error {
        &self.error
    }
}

impl From<&str> for ForcError {
    fn from(value: &str) -> Self {
        Self {
            error: anyhow::anyhow!("{value}"),
            exit_code: DEFAULT_ERROR_EXIT_CODE,
        }
    }
}

impl From<anyhow::Error> for ForcError {
    fn from(value: anyhow::Error) -> Self {
        Self {
            error: value,
            exit_code: DEFAULT_ERROR_EXIT_CODE,
        }
    }
}

impl From<std::io::Error> for ForcError {
    fn from(value: std::io::Error) -> Self {
        Self {
            error: value.into(),
            exit_code: DEFAULT_ERROR_EXIT_CODE,
        }
    }
}

impl Display for ForcError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.error.fmt(f)
    }
}

impl<T> Termination for ForcCliResult<T> {
    fn report(self) -> std::process::ExitCode {
        match self.result {
            Ok(_) => DEFAULT_SUCCESS_EXIT_CODE.into(),
            Err(e) => {
                error!("Error: {}", e);
                e.exit_code.into()
            }
        }
    }
}

impl<T> From<ForcResult<T>> for ForcCliResult<T> {
    fn from(value: ForcResult<T>) -> Self {
        Self { result: value }
    }
}

#[macro_export]
macro_rules! forc_result_bail {
    ($msg:literal $(,)?) => {
        return $crate::ForcResult::Err(anyhow::anyhow!($msg).into())
    };
    ($err:expr $(,)?) => {
        return $crate::ForcResult::Err(anyhow::anyhow!($err).into())
    };
    ($fmt:expr, $($arg:tt)*) => {
        return $crate::ForcResult::Err(anyhow::anyhow!($fmt, $($arg)*).into())
    };
}

#[cfg(feature = "fuel-tx")]
pub mod tx_utils {

    use anyhow::Result;
    use clap::Args;
    use serde::{Deserialize, Serialize};
    use sway_core::fuel_prelude::fuel_tx;

    /// Added salt used to derive the contract ID.
    #[derive(Debug, Args, Default, Deserialize, Serialize)]
    pub struct Salt {
        /// Added salt used to derive the contract ID.
        ///
        /// By default, this is `0x0000000000000000000000000000000000000000000000000000000000000000`.
        #[clap(long = "salt")]
        pub salt: Option<fuel_tx::Salt>,
    }

    /// Format `Log` and `LogData` receipts.
    pub fn format_log_receipts(
        receipts: &[fuel_tx::Receipt],
        pretty_print: bool,
    ) -> Result<String> {
        let mut receipt_to_json_array = serde_json::to_value(receipts)?;
        for (rec_index, receipt) in receipts.iter().enumerate() {
            let rec_value = receipt_to_json_array.get_mut(rec_index).ok_or_else(|| {
                anyhow::anyhow!(
                    "Serialized receipts does not contain {} th index",
                    rec_index
                )
            })?;
            match receipt {
                fuel_tx::Receipt::LogData {
                    data: Some(data), ..
                } => {
                    if let Some(v) = rec_value.pointer_mut("/LogData/data") {
                        *v = hex::encode(data).into();
                    }
                }
                fuel_tx::Receipt::ReturnData {
                    data: Some(data), ..
                } => {
                    if let Some(v) = rec_value.pointer_mut("/ReturnData/data") {
                        *v = hex::encode(data).into();
                    }
                }
                _ => {}
            }
        }
        if pretty_print {
            Ok(serde_json::to_string_pretty(&receipt_to_json_array)?)
        } else {
            Ok(serde_json::to_string(&receipt_to_json_array)?)
        }
    }
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
pub fn user_forc_directory() -> PathBuf {
    dirs::home_dir()
        .expect("unable to find the user home directory")
        .join(constants::USER_FORC_DIRECTORY)
}

/// The location at which `forc` will checkout git repositories.
pub fn git_checkouts_directory() -> PathBuf {
    user_forc_directory().join("git").join("checkouts")
}

/// Given a path to a directory we wish to lock, produce a path for an associated lock file.
///
/// Note that the lock file itself is simply a placeholder for co-ordinating access. As a result,
/// we want to create the lock file if it doesn't exist, but we can never reliably remove it
/// without risking invalidation of an existing lock. As a result, we use a dedicated, hidden
/// directory with a lock file named after the checkout path.
///
/// Note: This has nothing to do with `Forc.lock` files, rather this is about fd locks for
/// coordinating access to particular paths (e.g. git checkout directories).
fn fd_lock_path(path: &Path) -> PathBuf {
    const LOCKS_DIR_NAME: &str = ".locks";
    const LOCK_EXT: &str = "forc-lock";
    let file_name = hash_path(path);
    user_forc_directory()
        .join(LOCKS_DIR_NAME)
        .join(file_name)
        .with_extension(LOCK_EXT)
}

/// Constructs the path for the "dirty" flag file corresponding to the specified file.
///
/// This function uses a hashed representation of the original path for uniqueness.
pub fn is_dirty_path(path: &Path) -> PathBuf {
    const LOCKS_DIR_NAME: &str = ".lsp-locks";
    const LOCK_EXT: &str = "dirty";
    let file_name = hash_path(path);
    user_forc_directory()
        .join(LOCKS_DIR_NAME)
        .join(file_name)
        .with_extension(LOCK_EXT)
}

/// Hash the path to produce a file-system friendly file name.
/// Append the file stem for improved readability.
fn hash_path(path: &Path) -> String {
    let mut hasher = hash_map::DefaultHasher::default();
    path.hash(&mut hasher);
    let hash = hasher.finish();
    let file_name = match path.file_stem().and_then(|s| s.to_str()) {
        None => format!("{hash:X}"),
        Some(stem) => format!("{hash:X}-{stem}"),
    };
    file_name
}

/// Create an advisory lock over the given path.
///
/// See [fd_lock_path] for details.
pub fn path_lock(path: &Path) -> Result<fd_lock::RwLock<File>> {
    let lock_path = fd_lock_path(path);
    let lock_dir = lock_path
        .parent()
        .expect("lock path has no parent directory");
    std::fs::create_dir_all(lock_dir).context("failed to create forc advisory lock directory")?;
    let lock_file = File::create(&lock_path).context("failed to create advisory lock file")?;
    Ok(fd_lock::RwLock::new(lock_file))
}

pub fn program_type_str(ty: &TreeType) -> &'static str {
    match ty {
        TreeType::Script {} => "script",
        TreeType::Contract {} => "contract",
        TreeType::Predicate {} => "predicate",
        TreeType::Library { .. } => "library",
    }
}

pub fn print_compiling(ty: Option<&TreeType>, name: &str, src: &dyn std::fmt::Display) {
    // NOTE: We can only print the program type if we can parse the program, so
    // program type must be optional.
    let ty = match ty {
        Some(ty) => format!("{} ", program_type_str(ty)),
        None => "".to_string(),
    };
    tracing::info!(
        " {} {ty}{} ({src})",
        Colour::Green.bold().paint("Compiling"),
        ansi_term::Style::new().bold().paint(name)
    );
}

pub fn print_warnings(
    source_engine: &SourceEngine,
    terse_mode: bool,
    proj_name: &str,
    warnings: &[CompileWarning],
    tree_type: &TreeType,
) {
    if warnings.is_empty() {
        return;
    }
    let type_str = program_type_str(tree_type);

    if !terse_mode {
        warnings
            .iter()
            .for_each(|w| format_diagnostic(&w.to_diagnostic(source_engine)));
    }

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

pub fn print_on_failure(
    source_engine: &SourceEngine,
    terse_mode: bool,
    warnings: &[CompileWarning],
    errors: &[CompileError],
    reverse_results: bool,
) {
    let e_len = errors.len();
    let w_len = warnings.len();

    if !terse_mode {
        if reverse_results {
            warnings
                .iter()
                .rev()
                .for_each(|w| format_diagnostic(&w.to_diagnostic(source_engine)));
            errors
                .iter()
                .rev()
                .for_each(|e| format_diagnostic(&e.to_diagnostic(source_engine)));
        } else {
            warnings
                .iter()
                .for_each(|w| format_diagnostic(&w.to_diagnostic(source_engine)));
            errors
                .iter()
                .for_each(|e| format_diagnostic(&e.to_diagnostic(source_engine)));
        }
    }

    if e_len == 0 && w_len > 0 {
        println_red_err(&format!(
            "  Aborting. {} warning(s) treated as error(s).",
            warnings.len()
        ));
    } else {
        println_red_err(&format!(
            "  Aborting due to {} {}.",
            e_len,
            if e_len > 1 { "errors" } else { "error" }
        ));
    }
}

fn format_diagnostic(diagnostic: &Diagnostic) {
    /// Temporary switch for testing the feature.
    /// Keep it false until we decide to fully support the diagnostic codes.
    const SHOW_DIAGNOSTIC_CODE: bool = false;

    if diagnostic.is_old_style() {
        format_old_style_diagnostic(diagnostic.issue());
        return;
    }

    let mut label = String::new();
    get_title_label(diagnostic, &mut label);

    let snippet_title = Some(Annotation {
        label: Some(label.as_str()),
        id: if SHOW_DIAGNOSTIC_CODE {
            diagnostic.reason().map(|reason| reason.code())
        } else {
            None
        },
        annotation_type: diagnostic_level_to_annotation_type(diagnostic.level()),
    });

    let mut snippet_slices = Vec::<Slice<'_>>::new();

    // We first display labels from the issue file...
    if diagnostic.issue().is_in_source() {
        snippet_slices.push(construct_slice(diagnostic.labels_in_issue_source()))
    }

    // ...and then all the remaining labels from the other files.
    for source_path in diagnostic.related_sources(false) {
        snippet_slices.push(construct_slice(diagnostic.labels_in_source(source_path)))
    }

    let mut snippet_footer = Vec::<Annotation<'_>>::new();
    for help in diagnostic.help() {
        snippet_footer.push(Annotation {
            id: None,
            label: Some(help),
            annotation_type: AnnotationType::Help,
        });
    }

    let snippet = Snippet {
        title: snippet_title,
        slices: snippet_slices,
        footer: snippet_footer,
        opt: FormatOptions {
            color: true,
            ..Default::default()
        },
    };

    match diagnostic.level() {
        Level::Warning => tracing::warn!("{}\n____\n", DisplayList::from(snippet)),
        Level::Error => tracing::error!("{}\n____\n", DisplayList::from(snippet)),
    }

    fn format_old_style_diagnostic(issue: &Issue) {
        let annotation_type = label_type_to_annotation_type(issue.label_type());

        let snippet_title = Some(Annotation {
            label: if issue.is_in_source() {
                None
            } else {
                Some(issue.friendly_text())
            },
            id: None,
            annotation_type,
        });

        let mut snippet_slices = vec![];
        if issue.is_in_source() {
            let span = issue.span();
            let input = span.input();
            let mut start_pos = span.start();
            let mut end_pos = span.end();
            let (mut start, end) = span.line_col();
            let input = construct_window(&mut start, end, &mut start_pos, &mut end_pos, input);

            let slice = Slice {
                source: input,
                line_start: start.line,
                // Safe unwrap because the issue is in source, so the source path surely exists.
                origin: Some(issue.source_path().unwrap().as_str()),
                fold: false,
                annotations: vec![SourceAnnotation {
                    label: issue.friendly_text(),
                    annotation_type,
                    range: (start_pos, end_pos),
                }],
            };

            snippet_slices.push(slice);
        }

        let snippet = Snippet {
            title: snippet_title,
            footer: vec![],
            slices: snippet_slices,
            opt: FormatOptions {
                color: true,
                ..Default::default()
            },
        };

        tracing::error!("{}\n____\n", DisplayList::from(snippet));
    }

    fn get_title_label(diagnostics: &Diagnostic, label: &mut String) {
        label.clear();
        if let Some(reason) = diagnostics.reason() {
            label.push_str(reason.description());
        }
    }

    fn diagnostic_level_to_annotation_type(level: Level) -> AnnotationType {
        match level {
            Level::Warning => AnnotationType::Warning,
            Level::Error => AnnotationType::Error,
        }
    }
}

fn construct_slice(labels: Vec<&Label>) -> Slice {
    debug_assert!(
        !labels.is_empty(),
        "To construct slices, at least one label must be provided."
    );

    debug_assert!(
        labels.iter().all(|label| label.is_in_source()),
        "Slices can be constructed only for labels that are related to a place in source code."
    );

    debug_assert!(
        HashSet::<&str>::from_iter(labels.iter().map(|label| label.source_path().unwrap().as_str())).len() == 1,
        "Slices can be constructed only for labels that are related to places in the same source code."
    );

    let soruce_file = labels[0].source_path().map(|path| path.as_str());
    let source_code = labels[0].span().input();

    // Joint span of the code snippet that covers all the labels.
    let span = Span::join_all(labels.iter().map(|label| label.span().clone()));

    let (source, line_start, shift_in_bytes) = construct_code_snippet(&span, source_code);

    let mut annotations = vec![];

    for message in labels {
        annotations.push(SourceAnnotation {
            label: message.friendly_text(),
            annotation_type: label_type_to_annotation_type(message.label_type()),
            range: get_annotation_range(message.span(), source_code, shift_in_bytes),
        });
    }

    return Slice {
        source,
        line_start,
        origin: soruce_file,
        fold: true,
        annotations,
    };

    fn get_annotation_range(
        span: &Span,
        source_code: &str,
        shift_in_bytes: usize,
    ) -> (usize, usize) {
        let mut start_pos = span.start();
        let mut end_pos = span.end();

        let start_ix_bytes = start_pos - std::cmp::min(shift_in_bytes, start_pos);
        let end_ix_bytes = end_pos - std::cmp::min(shift_in_bytes, end_pos);

        // We want the start_pos and end_pos in terms of chars and not bytes, so translate.
        start_pos = source_code[shift_in_bytes..(shift_in_bytes + start_ix_bytes)]
            .chars()
            .count();
        end_pos = source_code[shift_in_bytes..(shift_in_bytes + end_ix_bytes)]
            .chars()
            .count();

        (start_pos, end_pos)
    }
}

fn label_type_to_annotation_type(label_type: LabelType) -> AnnotationType {
    match label_type {
        LabelType::Info => AnnotationType::Info,
        LabelType::Help => AnnotationType::Help,
        LabelType::Warning => AnnotationType::Warning,
        LabelType::Error => AnnotationType::Error,
    }
}

/// Given the overall span to be shown in the code snippet, determines how much of the input source
/// to show in the snippet.
///
/// Returns the source to be shown, the line start, and the offset of the snippet in bytes relative
/// to the begining of the input code.
///
/// The library we use doesn't handle auto-windowing and line numbers, so we must manually
/// calculate the line numbers and match them up with the input window. It is a bit fiddly.
fn construct_code_snippet<'a>(span: &Span, input: &'a str) -> (&'a str, usize, usize) {
    // how many lines to prepend or append to the highlighted region in the window
    const NUM_LINES_BUFFER: usize = 2;

    let (start, end) = span.line_col();

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

    (
        &input[calculated_start_ix..calculated_end_ix],
        lines_to_start_of_snippet,
        calculated_start_ix,
    )
}

// TODO: Remove once "old-style" diagnostic is fully replaced with new one and the backward
//       compatibility is no longer needed.
/// Given a start and an end position and an input, determine how much of a window to show in the
/// error.
/// Mutates the start and end indexes to be in line with the new slice length.
///
/// The library we use doesn't handle auto-windowing and line numbers, so we must manually
/// calculate the line numbers and match them up with the input window. It is a bit fiddly.
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
