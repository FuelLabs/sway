#![allow(dead_code)]
use crate::core::token::{AstToken, Token, TokenMap, TypedAstToken};
use crate::utils::{common::get_range_from_span, token};
use sway_core::language::{
    parsed::{Expression, ExpressionKind},
    Literal,
};
use sway_types::{Ident, Spanned};
use tower_lsp::lsp_types::{Diagnostic, DiagnosticSeverity};

pub(crate) fn generate_warnings_non_typed_tokens(tokens: &TokenMap) -> Vec<Diagnostic> {
    let warnings = tokens
        .iter()
        .filter(|item| {
            let ((_, _), token) = item.pair();
            token.typed.is_none()
        })
        .map(|item| {
            let (ident, _) = item.key();
            warning_from_ident(ident)
        })
        .collect();

    warnings
}

pub(crate) fn generate_warnings_for_parsed_tokens(tokens: &TokenMap) -> Vec<Diagnostic> {
    let warnings = tokens
        .iter()
        .map(|item| {
            let (ident, _) = item.key();
            warning_from_ident(ident)
        })
        .collect();

    warnings
}

pub(crate) fn generate_warnings_for_typed_tokens(tokens: &TokenMap) -> Vec<Diagnostic> {
    let warnings = tokens
        .iter()
        .filter(|item| {
            let ((_, _), token) = item.pair();
            token.typed.is_some()
        })
        .map(|item| {
            let (ident, _) = item.key();
            warning_from_ident(ident)
        })
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

pub(crate) fn debug_print_ident_and_token(ident: &Ident, token: &Token) {
    let pos = ident.span().start_pos().line_col();
    let line_num = pos.0 as u32;

    tracing::debug!(
        "line num = {:?} | name: = {:?} | ast_node_type = {:?} | type_id = {:?}",
        line_num,
        ident.as_str(),
        ast_node_type(token),
        token::type_id(token),
    );
}

fn ast_node_type(token_type: &Token) -> String {
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
        },
        None => match &token_type.parsed {
            AstToken::Expression(Expression {
                kind: ExpressionKind::Literal(value),
                ..
            }) => literal_to_string(value),
            _ => "".to_string(),
        },
    }
}

fn literal_to_string(literal: &Literal) -> String {
    match literal {
        Literal::U8(_) => "u8".into(),
        Literal::U16(_) => "u16".into(),
        Literal::U32(_) => "u32".into(),
        Literal::U64(_) => "u64".into(),
        Literal::Numeric(_) => "u64".into(),
        Literal::String(len) => format!("str[{}]", len.as_str().len()),
        Literal::Boolean(_) => "bool".into(),
        Literal::B256(_) => "b256".into(),
    }
}
