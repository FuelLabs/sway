#![allow(dead_code)]
use crate::core::token::{Token, TokenIdent};
use dashmap::mapref::multiple::RefMulti;
use lsp_types::{Diagnostic, DiagnosticSeverity};
use sway_core::{
    decl_engine::DeclEngine,
    language::{ty, Literal},
};

pub(crate) fn generate_warnings_non_typed_tokens<'s, I>(tokens: I) -> Vec<Diagnostic>
where
    I: Iterator<Item = RefMulti<'s, TokenIdent, Token>>,
{
    tokens
        .filter_map(|entry| {
            let (ident, token) = entry.pair();
            if token.as_parsed().is_some() {
                Some(warning_from_ident(ident))
            } else {
                None
            }
        })
        .collect()
}

pub(crate) fn generate_warnings_for_parsed_tokens<'s, I>(tokens: I) -> Vec<Diagnostic>
where
    I: Iterator<Item = RefMulti<'s, TokenIdent, Token>>,
{
    tokens
        .map(|entry| warning_from_ident(entry.key()))
        .collect()
}

pub(crate) fn generate_warnings_for_typed_tokens<'s, I>(tokens: I) -> Vec<Diagnostic>
where
    I: Iterator<Item = RefMulti<'s, TokenIdent, Token>>,
{
    tokens
        .filter_map(|entry| {
            let (ident, token) = entry.pair();
            if token.as_typed().is_some() {
                Some(warning_from_ident(ident))
            } else {
                None
            }
        })
        .collect()
}

fn warning_from_ident(ident: &TokenIdent) -> Diagnostic {
    Diagnostic {
        range: ident.range,
        severity: Some(DiagnosticSeverity::WARNING),
        message: String::new(),
        ..Default::default()
    }
}

fn literal_to_string(literal: &Literal) -> String {
    match literal {
        Literal::U8(_) => "u8".into(),
        Literal::U16(_) => "u16".into(),
        Literal::U32(_) => "u32".into(),
        Literal::U64(_) | Literal::Numeric(_) => "u64".into(),
        Literal::U256(_) => "u256".into(),
        Literal::String(len) => format!("str[{}]", len.as_str().len()),
        Literal::Boolean(_) => "bool".into(),
        Literal::B256(_) => "b256".into(),
    }
}

/// Print the AST nodes in a human readable format
/// by getting the types from the declaration engine
/// and formatting them into a String.
pub(crate) fn print_decl_engine_types(
    all_nodes: &[ty::TyAstNode],
    decl_engine: &DeclEngine,
) -> String {
    all_nodes
        .iter()
        .map(|n| match &n.content {
            ty::TyAstNodeContent::Declaration(declaration) => match declaration {
                ty::TyDecl::ConstantDecl(ty::ConstantDecl { decl_id, .. }) => {
                    let const_decl = decl_engine.get_constant(decl_id);
                    format!("{const_decl:#?}")
                }
                ty::TyDecl::FunctionDecl(ty::FunctionDecl { decl_id, .. }) => {
                    let func_decl = decl_engine.get_function(decl_id);
                    format!("{func_decl:#?}")
                }
                ty::TyDecl::TraitDecl(ty::TraitDecl { decl_id, .. }) => {
                    let trait_decl = decl_engine.get_trait(decl_id);
                    format!("{trait_decl:#?}")
                }
                ty::TyDecl::StructDecl(ty::StructDecl { decl_id, .. }) => {
                    let struct_decl = decl_engine.get_struct(decl_id);
                    format!("{struct_decl:#?}")
                }
                ty::TyDecl::EnumDecl(ty::EnumDecl { decl_id, .. }) => {
                    let enum_decl = decl_engine.get_enum(decl_id);
                    format!("{enum_decl:#?}")
                }
                ty::TyDecl::AbiDecl(ty::AbiDecl { decl_id, .. }) => {
                    let abi_decl = decl_engine.get_abi(decl_id);
                    format!("{abi_decl:#?}")
                }
                ty::TyDecl::StorageDecl(ty::StorageDecl { decl_id, .. }) => {
                    let storage_decl = decl_engine.get_storage(decl_id);
                    format!("{storage_decl:#?}")
                }
                _ => format!("{declaration:#?}"),
            },
            ty::TyAstNodeContent::Expression(expression) => {
                format!("{expression:#?}")
            }
            ty::TyAstNodeContent::Statement(statement) => format!("{statement:#?}"),
            ty::TyAstNodeContent::SideEffect(side_effect) => format!("{side_effect:#?}"),
            ty::TyAstNodeContent::Error(_, _) => "error".to_string(),
        })
        .fold(String::new(), |output, s| format!("{output}{s}\n"))
}
