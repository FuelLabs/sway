use tower_lsp::lsp_types::{Diagnostic, DiagnosticSeverity, Position, Range};

use sway_core::{CompileError, CompileWarning};

pub fn get_diagnostics(
    warnings: Vec<CompileWarning>,
    errors: Vec<CompileError>,
) -> Vec<Diagnostic> {
    let errors: Vec<Diagnostic> = errors
        .iter()
        .map(|error| {
            let range = get_range(&WarningOrError::Error(error));
            Diagnostic {
                range,
                severity: Some(DiagnosticSeverity::ERROR),
                message: error.to_friendly_error_string(),
                ..Default::default()
            }
        })
        .collect();

    let warnings: Vec<Diagnostic> = warnings
        .iter()
        .map(|warning| {
            let range = get_range(&WarningOrError::Warning(warning));
            Diagnostic {
                range,
                severity: Some(DiagnosticSeverity::WARNING),
                message: warning.to_friendly_warning_string(),
                ..Default::default()
            }
        })
        .collect();

    vec![warnings, errors].into_iter().flatten().collect()
}

fn get_range(warning_or_error: &WarningOrError<'_>) -> Range {
    let (start, end) = match warning_or_error {
        WarningOrError::Error(error) => error.line_col(),
        WarningOrError::Warning(warning) => warning.line_col(),
    };

    let start_line = start.line as u32 - 1;
    let start_character = start.col as u32;

    let end_line = end.line as u32 - 1;
    let end_character = end.col as u32;

    Range {
        start: Position::new(start_line, start_character),
        end: Position::new(end_line, end_character),
    }
}

enum WarningOrError<'s> {
    Warning(&'s CompileWarning),
    Error(&'s CompileError),
}
