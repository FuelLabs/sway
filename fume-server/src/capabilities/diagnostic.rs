use lsp::{Diagnostic, DiagnosticSeverity, Position, Range};
use lspower::lsp::{self};

use parser::{CompileError, CompileResult, CompileWarning};

pub fn perform_diagnostics(text_document: &str) -> Option<Vec<Diagnostic>> {
    match parser::parse(text_document) {
        CompileResult::Err { warnings, errors } => {
            let errors: Vec<Diagnostic> = errors
                .iter()
                .map(|error| {
                    let range = get_range(&WarningOrError::Error(error));
                    Diagnostic {
                        range,
                        severity: Some(DiagnosticSeverity::Error),
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
                        severity: Some(DiagnosticSeverity::Warning),
                        message: warning.to_friendly_warning_string(),
                        ..Default::default()
                    }
                })
                .collect();

            return Some(vec![warnings, errors].into_iter().flatten().collect());
        }
        _ => None,
    }
}

fn get_range<'s>(warning_or_error: &WarningOrError<'s>) -> Range {
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
    Warning(&'s CompileWarning<'s>),
    Error(&'s CompileError<'s>),
}
