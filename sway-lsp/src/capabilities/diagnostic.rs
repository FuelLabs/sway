use tower_lsp::lsp_types::{Diagnostic, DiagnosticSeverity, Position, Range};

use sway_error::error::CompileError;
use sway_error::warning::CompileWarning;
use sway_types::{LineCol, Spanned};

#[derive(Debug)]
pub struct Diagnostics {
    pub warnings: Vec<Diagnostic>,
    pub errors: Vec<Diagnostic>,
}

fn get_error_diagnostics(errors: &[CompileError]) -> Vec<Diagnostic> {
    Vec::from_iter(errors.iter().map(|error| Diagnostic {
        range: get_range(error.span().line_col()),
        severity: Some(DiagnosticSeverity::ERROR),
        message: format!("{}", error),
        ..Default::default()
    }))
}

fn get_warning_diagnostics(warnings: &[CompileWarning]) -> Vec<Diagnostic> {
    Vec::from_iter(warnings.iter().map(|warning| Diagnostic {
        range: get_range(warning.span().line_col()),
        severity: Some(DiagnosticSeverity::WARNING),
        message: warning.to_friendly_warning_string(),
        ..Default::default()
    }))
}

pub fn get_diagnostics(warnings: &[CompileWarning], errors: &[CompileError]) -> Diagnostics {
    Diagnostics {
        warnings: get_warning_diagnostics(warnings),
        errors: get_error_diagnostics(errors),
    }
}

fn get_range((start, end): (LineCol, LineCol)) -> Range {
    let pos = |lc: LineCol| Position::new(lc.line as u32 - 1, lc.col as u32 - 1);
    let start = pos(start);
    let end = pos(end);
    Range { start, end }
}
