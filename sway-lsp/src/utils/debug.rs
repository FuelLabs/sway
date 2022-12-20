#![allow(dead_code)]
use crate::core::token::{get_range_from_span, Token};
use sway_core::language::Literal;
use sway_types::{Ident, Spanned};
use tower_lsp::lsp_types::{Diagnostic, DiagnosticSeverity};

pub(crate) fn generate_warnings_non_typed_tokens<I>(tokens: I) -> Vec<Diagnostic>
where
    I: Iterator<Item = (Ident, Token)>,
{
    tokens
        .filter(|(_, token)| token.typed.is_none())
        .map(|(ident, _)| warning_from_ident(&ident))
        .collect()
}

pub(crate) fn generate_warnings_for_parsed_tokens<I>(tokens: I) -> Vec<Diagnostic>
where
    I: Iterator<Item = (Ident, Token)>,
{
    tokens
        .map(|(ident, _)| warning_from_ident(&ident))
        .collect()
}

pub(crate) fn generate_warnings_for_typed_tokens<I>(tokens: I) -> Vec<Diagnostic>
where
    I: Iterator<Item = (Ident, Token)>,
{
    tokens
        .filter(|(_, token)| token.typed.is_some())
        .map(|(ident, _)| warning_from_ident(&ident))
        .collect()
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
