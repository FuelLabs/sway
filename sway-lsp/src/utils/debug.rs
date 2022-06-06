use crate::core::{
    token::Token,
    traverse_typed_tree::get_type_id,
    typed_token_type::{TokenMap, TokenType},
};
use crate::utils::common::get_range_from_span;
use sway_types::Ident;
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

pub fn debug_print_ident_and_token(ident: &Ident, token: &TokenType) {
    let pos = ident.span().start_pos().line_col();
    let line_num = pos.0 as u32;

    tracing::info!(
        "line num = {:?} | name: = {:?} | ast_node_type = {:?} | type_id = {:?}",
        line_num,
        ident.as_str(),
        ast_node_type(token),
        get_type_id(token),
    );
}

fn ast_node_type(token: &TokenType) -> String {
    match &token {
        TokenType::TypedDeclaration(dec) => dec.friendly_name().to_string(),
        TokenType::TypedExpression(exp) => exp.expression.pretty_print(),
        TokenType::TypedFunctionParameter(_) => "function parameter".to_string(),
        TokenType::TypedStructField(_) => "struct field".to_string(),
        TokenType::TypedEnumVariant(_) => "enum variant".to_string(),
        TokenType::TypedTraitFn(_) => "trait function".to_string(),
        TokenType::TypedStorageField(_) => "storage field".to_string(),
        TokenType::TypeCheckedStorageReassignDescriptor(_) => {
            "storage reassignment descriptor".to_string()
        }
        TokenType::TypedReassignment(_) => "reassignment".to_string(),
        _ => "".to_string(),
    }
}
