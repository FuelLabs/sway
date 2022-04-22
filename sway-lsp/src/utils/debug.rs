use crate::core::token::Token;
use tower_lsp::lsp_types::{Diagnostic, DiagnosticSeverity};

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
