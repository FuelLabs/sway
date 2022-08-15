use crate::core::token::{AstToken, Token, TokenMap, TypedAstToken};
use crate::utils::common::get_range_from_span;
use sway_core::{
    semantic_analysis::ast_node::{
        expression::typed_expression_variant::TypedExpressionVariant, TypedDeclaration,
    },
    Declaration, ExpressionKind, Literal,
};
use sway_types::{Ident, Spanned};
use tower_lsp::lsp_types::{Location, SymbolInformation, SymbolKind, Url};

pub fn to_symbol_information(token_map: &TokenMap, url: Url) -> Vec<SymbolInformation> {
    let mut symbols: Vec<SymbolInformation> = vec![];

    for item in token_map.iter() {
        let ((ident, _), token) = item.pair();
        let symbol = symbol_info(ident, token, url.clone());
        symbols.push(symbol)
    }

    symbols
}

#[allow(warnings)]
// TODO: the "deprecated: None" field is deprecated according to this library
fn symbol_info(ident: &Ident, token: &Token, url: Url) -> SymbolInformation {
    let range = get_range_from_span(&ident.span());
    SymbolInformation {
        name: ident.as_str().to_string(),
        kind: {
            match &token.typed {
                Some(typed_token) => typed_to_symbol_kind(&typed_token),
                None => parsed_to_symbol_kind(&token.parsed),
            }
        },
        location: Location::new(url, range),
        tags: None,
        container_name: None,
        deprecated: None,
    }
}

fn parsed_to_symbol_kind(ast_token: &AstToken) -> SymbolKind {
    match ast_token {
        AstToken::Declaration(dec) => {
            match dec {
                Declaration::VariableDeclaration(_) => SymbolKind::VARIABLE,
                Declaration::FunctionDeclaration(_) => SymbolKind::FUNCTION,
                Declaration::TraitDeclaration(_) => SymbolKind::INTERFACE,
                Declaration::StructDeclaration(_) => SymbolKind::STRUCT,
                Declaration::EnumDeclaration(_) => SymbolKind::ENUM,
                Declaration::ConstantDeclaration(_) => SymbolKind::CONSTANT,
                Declaration::ImplTrait { .. } => SymbolKind::INTERFACE,
                Declaration::AbiDeclaration(_) => SymbolKind::INTERFACE,
                // currently we return `variable` type as default
                Declaration::Reassignment(_)
                | Declaration::ImplSelf { .. }
                | Declaration::StorageDeclaration(_)
                | Declaration::Break { .. }
                | Declaration::Continue { .. } => SymbolKind::VARIABLE,
            }
        }
        AstToken::Expression(exp) => {
            match &exp.kind {
                ExpressionKind::Literal(value) => match value {
                    Literal::String(_) => SymbolKind::STRING,
                    Literal::Boolean(_) => SymbolKind::BOOLEAN,
                    _ => SymbolKind::NUMBER,
                },
                ExpressionKind::FunctionApplication(_) => SymbolKind::FUNCTION,
                ExpressionKind::Variable(_) => SymbolKind::VARIABLE,
                ExpressionKind::Array(_) => SymbolKind::ARRAY,
                ExpressionKind::Struct(_) => SymbolKind::STRUCT,
                // currently we return `variable` type as default
                _ => SymbolKind::VARIABLE,
            }
        }
        AstToken::FunctionDeclaration(_) => SymbolKind::FUNCTION,
        AstToken::FunctionParameter(_) => SymbolKind::TYPE_PARAMETER,
        AstToken::StructField(_) => SymbolKind::FIELD,
        AstToken::EnumVariant(_) => SymbolKind::ENUM_MEMBER,
        AstToken::TraitFn(_) => SymbolKind::FUNCTION,
        AstToken::StorageField(_) => SymbolKind::FIELD,
        AstToken::Reassignment(_) => SymbolKind::VARIABLE,
    }
}

fn typed_to_symbol_kind(typed_ast_token: &TypedAstToken) -> SymbolKind {
    match typed_ast_token {
        TypedAstToken::TypedDeclaration(dec) => {
            match dec {
                TypedDeclaration::VariableDeclaration(_) => SymbolKind::VARIABLE,
                TypedDeclaration::ConstantDeclaration(_) => SymbolKind::CONSTANT,
                TypedDeclaration::FunctionDeclaration(_) => SymbolKind::FUNCTION,
                TypedDeclaration::TraitDeclaration(_) => SymbolKind::INTERFACE,
                TypedDeclaration::StructDeclaration(_) => SymbolKind::STRUCT,
                TypedDeclaration::EnumDeclaration(_) => SymbolKind::ENUM,
                TypedDeclaration::ImplTrait { .. } => SymbolKind::INTERFACE,
                TypedDeclaration::AbiDeclaration(_) => SymbolKind::INTERFACE,
                TypedDeclaration::GenericTypeForFunctionScope { .. } => SymbolKind::TYPE_PARAMETER,
                // currently we return `variable` type as default
                TypedDeclaration::Reassignment(_)
                | TypedDeclaration::ErrorRecovery
                | TypedDeclaration::StorageDeclaration(_)
                | TypedDeclaration::StorageReassignment(_)
                | TypedDeclaration::Break { .. }
                | TypedDeclaration::Continue { .. } => SymbolKind::VARIABLE,
            }
        }
        TypedAstToken::TypedExpression(exp) => {
            match &exp.expression {
                TypedExpressionVariant::Literal(lit) => match lit {
                    Literal::String(_) => SymbolKind::STRING,
                    Literal::Boolean(_) => SymbolKind::BOOLEAN,
                    _ => SymbolKind::NUMBER,
                },
                TypedExpressionVariant::FunctionApplication { .. } => SymbolKind::FUNCTION,
                TypedExpressionVariant::VariableExpression { .. } => SymbolKind::VARIABLE,
                TypedExpressionVariant::Array { .. } => SymbolKind::ARRAY,
                TypedExpressionVariant::StructExpression { .. } => SymbolKind::STRUCT,
                TypedExpressionVariant::StructFieldAccess { .. } => SymbolKind::FIELD,
                // currently we return `variable` type as default
                _ => SymbolKind::VARIABLE,
            }
        }
        TypedAstToken::TypedFunctionDeclaration(_) => SymbolKind::FUNCTION,
        TypedAstToken::TypedFunctionParameter(_) => SymbolKind::TYPE_PARAMETER,
        TypedAstToken::TypedStructField(_) => SymbolKind::FIELD,
        TypedAstToken::TypedEnumVariant(_) => SymbolKind::ENUM_MEMBER,
        TypedAstToken::TypedTraitFn(_) => SymbolKind::FUNCTION,
        TypedAstToken::TypedStorageField(_) => SymbolKind::FIELD,
        TypedAstToken::TypeCheckedStorageReassignDescriptor(_)
        | TypedAstToken::TypedReassignment(_) => SymbolKind::VARIABLE,
    }
}
