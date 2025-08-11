use std::collections::HashMap;
use std::path::PathBuf;

use lsp_types::{Diagnostic, DiagnosticSeverity, DiagnosticTag, Position, Range};
use serde::{Deserialize, Serialize};
use sway_error::warning::{CompileInfo, CompileWarning, Info};
use sway_error::{error::CompileError, warning::Warning};
use sway_types::{LineCol, LineColRange, SourceEngine, Spanned};

pub(crate) type DiagnosticMap = HashMap<PathBuf, Diagnostics>;

#[derive(Debug, Default, Clone)]
pub struct Diagnostics {
    pub infos: Vec<Diagnostic>,
    pub warnings: Vec<Diagnostic>,
    pub errors: Vec<Diagnostic>,
}

fn get_error_diagnostic(error: &CompileError) -> Diagnostic {
    let data = serde_json::to_value(DiagnosticData::try_from(error.clone()).ok()).ok();

    Diagnostic {
        range: get_range(error.span().line_col_one_index()),
        severity: Some(DiagnosticSeverity::ERROR),
        message: format!("{error}"),
        data,
        ..Default::default()
    }
}

fn get_warning_diagnostic(warning: &CompileWarning) -> Diagnostic {
    Diagnostic {
        range: get_range(warning.span().line_col_one_index()),
        severity: Some(DiagnosticSeverity::WARNING),
        message: warning.to_friendly_warning_string(),
        tags: get_warning_diagnostic_tags(&warning.warning_content),
        ..Default::default()
    }
}

fn get_info_diagnostic(info: &CompileInfo) -> Diagnostic {
    Diagnostic {
        range: get_range(info.span().line_col_one_index()),
        severity: Some(DiagnosticSeverity::INFORMATION),
        message: info.to_friendly_string(),
        tags: get_info_diagnostic_tags(&info.content),
        ..Default::default()
    }
}

pub fn get_diagnostics(
    infos: &[CompileInfo],
    warnings: &[CompileWarning],
    errors: &[CompileError],
    source_engine: &SourceEngine,
) -> DiagnosticMap {
    let mut diagnostics = DiagnosticMap::new();
    for info in infos {
        let diagnostic = get_info_diagnostic(info);
        if let Some(source_id) = info.span().source_id() {
            let path = source_engine.get_path(source_id);
            diagnostics.entry(path).or_default().infos.push(diagnostic);
        }
    }
    for warning in warnings {
        let diagnostic = get_warning_diagnostic(warning);
        if let Some(source_id) = warning.span().source_id() {
            let path = source_engine.get_path(source_id);
            diagnostics
                .entry(path)
                .or_default()
                .warnings
                .push(diagnostic);
        }
    }
    for error in errors {
        let diagnostic = get_error_diagnostic(error);
        if let Some(source_id) = error.span().source_id() {
            let path = source_engine.get_path(source_id);
            diagnostics.entry(path).or_default().errors.push(diagnostic);
        }
    }
    diagnostics
}

fn get_range(LineColRange { start, end }: LineColRange) -> Range {
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

fn get_info_diagnostic_tags(info: &Info) -> Option<Vec<DiagnosticTag>> {
    match info {
        Info::ImplTraitsForType { .. } => Some(vec![DiagnosticTag::UNNECESSARY]),
    }
}

/// Extra data to be sent with a diagnostic and provided in CodeAction context.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct DiagnosticData {
    pub unknown_symbol_name: Option<String>,
}

impl TryFrom<CompileWarning> for DiagnosticData {
    type Error = anyhow::Error;

    fn try_from(_value: CompileWarning) -> Result<Self, Self::Error> {
        anyhow::bail!("Not implemented");
    }
}

impl TryFrom<CompileError> for DiagnosticData {
    type Error = anyhow::Error;

    fn try_from(value: CompileError) -> Result<Self, Self::Error> {
        match value {
            CompileError::SymbolNotFound { name, .. } => Ok(DiagnosticData {
                unknown_symbol_name: Some(name.to_string()),
            }),
            CompileError::TraitNotFound { name, .. } => Ok(DiagnosticData {
                unknown_symbol_name: Some(name),
            }),
            CompileError::UnknownVariable { var_name, .. } => Ok(DiagnosticData {
                unknown_symbol_name: Some(var_name.to_string()),
            }),
            _ => anyhow::bail!("Not implemented"),
        }
    }
}
