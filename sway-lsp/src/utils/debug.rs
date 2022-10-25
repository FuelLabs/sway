#![allow(dead_code)]
use crate::core::token::TokenMap;
use crate::utils::common::get_range_from_span;
use sway_core::language::Literal;
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
