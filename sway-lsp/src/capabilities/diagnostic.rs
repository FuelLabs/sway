use std::collections::HashMap;
use std::path::PathBuf;

use lsp_types::{Diagnostic, DiagnosticSeverity, DiagnosticTag, Position, Range};
use sway_error::warning::CompileWarning;
use sway_error::{error::CompileError, warning::Warning};
use sway_types::{LineCol, MaybeSpanned, SourceEngine, Spanned};

pub type DiagnosticMap = HashMap<PathBuf, Diagnostics>;

#[derive(Debug, Default, Clone)]
pub struct Diagnostics {
    pub warnings: Vec<Diagnostic>,
    pub errors: Vec<Diagnostic>,
}

fn get_error_diagnostic(error: &CompileError) -> Diagnostic {
    Diagnostic {
        range: get_range(
            error
                .try_span()
                .map(|s| s.line_col())
                // FIXME: there has to be a better way to do this
                .unwrap_or((LineCol { line: 0, col: 0 }, LineCol { line: 0, col: 0 })),
        ),
        severity: Some(DiagnosticSeverity::ERROR),
        message: format!("{error}"),
        ..Default::default()
    }
}

fn get_warning_diagnostic(warning: &CompileWarning) -> Diagnostic {
    Diagnostic {
        range: get_range(warning.span().line_col()),
        severity: Some(DiagnosticSeverity::WARNING),
        message: warning.to_friendly_warning_string(),
        tags: get_warning_diagnostic_tags(&warning.warning_content),
        ..Default::default()
    }
}

pub fn get_diagnostics(
    warnings: &[CompileWarning],
    errors: &[CompileError],
    source_engine: &SourceEngine,
) -> DiagnosticMap {
    let mut diagnostics = DiagnosticMap::new();
    for warning in warnings {
        let diagnostic = get_warning_diagnostic(warning);
        if let Some(source_id) = warning.span().source_id() {
            let path = source_engine.get_path(source_id);
            diagnostics
                .entry(path)
                .or_insert_with(Diagnostics::default)
                .warnings
                .push(diagnostic);
        }
    }
    for error in errors {
        let diagnostic = get_error_diagnostic(error);
        if let Some(source_id) = error.try_span().and_then(|s| s.source_id().cloned()) {
            let path = source_engine.get_path(&source_id);
            diagnostics
                .entry(path)
                .or_insert_with(Diagnostics::default)
                .errors
                .push(diagnostic);
        }
    }

    diagnostics
}

fn get_range((start, end): (LineCol, LineCol)) -> Range {
    let pos = |lc: LineCol| Position::new(lc.line as u32 - 1, lc.col as u32 - 1);
    let start = pos(start);
    let end = pos(end);
    Range { start, end }
}

fn get_warning_diagnostic_tags(warning: &Warning) -> Option<Vec<DiagnosticTag>> {
    match warning {
        Warning::StructFieldNeverRead
        | Warning::DeadDeclaration
        | Warning::DeadEnumDeclaration
        | Warning::DeadEnumVariant { .. }
        | Warning::DeadFunctionDeclaration
        | Warning::DeadMethod
        | Warning::DeadStorageDeclaration
        | Warning::DeadStorageDeclarationForFunction { .. }
        | Warning::DeadStructDeclaration
        | Warning::DeadTrait
        | Warning::MatchExpressionUnreachableArm { .. }
        | Warning::UnreachableCode
        | Warning::UnusedReturnValue { .. } => Some(vec![DiagnosticTag::UNNECESSARY]),
        _ => None,
    }
}
