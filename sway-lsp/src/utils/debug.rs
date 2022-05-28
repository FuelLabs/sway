use crate::core::token::Token;
use tower_lsp::lsp_types::{Diagnostic, DiagnosticSeverity};
use crate::core::traverse_typed_tree::{get_range_from_span, TokenMap};
// Flags for debugging various parts of the server
#[derive(Debug, Default)]
pub struct DebugFlags {
    /// Instructs the client to draw squiggly lines
    /// under all of the tokens that our server managed to parse
    pub parsed_tokens_as_warnings: bool,
}

pub fn generate_warnings_for_parsed_tokens(tokens: &[Token]) -> Vec<Diagnostic> {
    let warnings = tokens
        .iter()
        .map(|token| Diagnostic {
            range: token.range,
            severity: Some(DiagnosticSeverity::WARNING),
            message: token.name.clone(),
            ..Default::default()
        })
        .collect();

    warnings
}

pub fn generate_warnings_for_typed_tokens(tokens: &TokenMap) -> Vec<Diagnostic> {
    let warnings = tokens
        .keys()
        .map(|(ident, span)| Diagnostic {
            range: get_range_from_span(span),
            severity: Some(DiagnosticSeverity::WARNING),
            message: ident.as_str().to_string(),
            ..Default::default()
        })
        .collect();

    warnings
}
