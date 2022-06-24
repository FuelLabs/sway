use crate::core::{
    token::{TokenMap, TokenType, TypedAstToken},
};
use crate::utils::{
    common::get_range_from_span,
    token::get_type_id,
};
use sway_types::{Ident, Spanned};
use tower_lsp::lsp_types::{Diagnostic, DiagnosticSeverity};

// Flags for debugging various parts of the server
#[derive(Debug, Default)]
pub struct DebugFlags {
    /// Instructs the client to draw squiggly lines
    /// under all of the tokens that our server managed to parse
    pub parsed_tokens_as_warnings: bool,
}

pub fn generate_warnings_for_parsed_tokens(tokens: &TokenMap) -> Vec<Diagnostic> {
    let warnings = tokens
        .iter()
        .filter(|(k,v)| {
            v.typed.is_none()
        })
        .map(|((ident,_), _)|
            warning_from_ident(ident)
        )
        .collect();

    warnings
}


pub fn generate_warnings_for_typed_tokens(tokens: &TokenMap) -> Vec<Diagnostic> {
    let warnings = tokens
        .iter()
        .filter(|(k,v)| {
            v.typed.is_some()
        })
        .map(|((ident,_), _)|
            warning_from_ident(ident)
        )
        .collect();

    warnings
}

fn warning_from_ident(ident: &Ident) -> Diagnostic {
    Diagnostic {
        range: get_range_from_span(&ident.span()),
        severity: Some(DiagnosticSeverity::WARNING),
        message: ident.as_str().to_string(),
        ..Default::default()
    }
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

fn ast_node_type(token_type: &TokenType) -> String {
    match &token_type.typed {
        Some(typed_ast_token) => match typed_ast_token {
            TypedAstToken::TypedDeclaration(dec) => dec.friendly_name().to_string(),
            TypedAstToken::TypedExpression(exp) => exp.expression.to_string(),
            TypedAstToken::TypedFunctionParameter(_) => "function parameter".to_string(),
            TypedAstToken::TypedStructField(_) => "struct field".to_string(),
            TypedAstToken::TypedEnumVariant(_) => "enum variant".to_string(),
            TypedAstToken::TypedTraitFn(_) => "trait function".to_string(),
            TypedAstToken::TypedStorageField(_) => "storage field".to_string(),
            TypedAstToken::TypeCheckedStorageReassignDescriptor(_) => {
                "storage reassignment descriptor".to_string()
            }
            TypedAstToken::TypedReassignment(_) => "reassignment".to_string(),
            _ => "".to_string(),
        }
        None => {
            "".to_string()
        }
    }
}
