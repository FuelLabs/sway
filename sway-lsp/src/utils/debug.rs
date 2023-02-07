#![allow(dead_code)]
use crate::core::token::{get_range_from_span, Token};
use sway_core::{
    decl_engine::DeclEngine,
    language::{ty, Literal},
};
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
                ty::TyDeclaration::ConstantDeclaration(decl_id) => {
                    let const_decl = decl_engine
                        .get_constant(decl_id.clone(), &decl_id.span())
                        .unwrap();
                    format!("{const_decl:#?}")
                }
                ty::TyDeclaration::FunctionDeclaration(decl_id) => {
                    let func_decl = decl_engine
                        .get_function(decl_id.clone(), &decl_id.span())
                        .unwrap();
                    format!("{func_decl:#?}")
                }
                ty::TyDeclaration::TraitDeclaration(decl_id) => {
                    let trait_decl = decl_engine
                        .get_trait(decl_id.clone(), &decl_id.span())
                        .unwrap();
                    format!("{trait_decl:#?}")
                }
                ty::TyDeclaration::StructDeclaration(decl_id) => {
                    let struct_decl = decl_engine
                        .get_struct(decl_id.clone(), &decl_id.span())
                        .unwrap();
                    format!("{struct_decl:#?}")
                }
                ty::TyDeclaration::EnumDeclaration(decl_id) => {
                    let enum_decl = decl_engine
                        .get_enum(decl_id.clone(), &decl_id.span())
                        .unwrap();
                    format!("{enum_decl:#?}")
                }
                ty::TyDeclaration::AbiDeclaration(decl_id) => {
                    let abi_decl = decl_engine
                        .get_abi(decl_id.clone(), &decl_id.span())
                        .unwrap();
                    format!("{abi_decl:#?}")
                }
                ty::TyDeclaration::StorageDeclaration(decl_id) => {
                    let storage_decl = decl_engine
                        .get_storage(decl_id.clone(), &decl_id.span())
                        .unwrap();
                    format!("{storage_decl:#?}")
                }
                _ => format!("{declaration:#?}"),
            },
            ty::TyAstNodeContent::Expression(expression)
            | ty::TyAstNodeContent::ImplicitReturnExpression(expression) => {
                format!("{expression:#?}")
            }
            ty::TyAstNodeContent::SideEffect(side_effect) => format!("{side_effect:#?}"),
        })
        .map(|s| format!("{s}\n"))
        .collect()
}
