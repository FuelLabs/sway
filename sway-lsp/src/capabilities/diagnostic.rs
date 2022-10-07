use tower_lsp::lsp_types::{Diagnostic, DiagnosticSeverity, Position, Range};

use sway_core::{error::LineCol, CompileError, CompileWarning};

pub fn get_diagnostics(
    warnings: Vec<CompileWarning>,
    errors: Vec<FailedToCompile>,
) -> Vec<Diagnostic> {
    let errors = errors.iter().map(|error| Diagnostic {
        range: get_range(error.line_col()),
        severity: Some(DiagnosticSeverity::ERROR),
        message: format!("{}", error),
        ..Default::default()
    });

    let warnings = warnings.iter().map(|warning| Diagnostic {
        range: get_range(warning.line_col()),
        severity: Some(DiagnosticSeverity::WARNING),
        message: warning.to_friendly_warning_string(),
        ..Default::default()
    });

    let mut all = errors.collect::<Vec<_>>();
    all.extend(warnings);
    all
}

fn get_range((start, end): (LineCol, LineCol)) -> Range {
    let pos = |lc: LineCol| Position::new(lc.line as u32 - 1, lc.col as u32 - 1);
    let start = pos(start);
    let end = pos(end);
    Range { start, end }
}
