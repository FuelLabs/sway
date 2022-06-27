use crate::core::token::{AstToken, TypedAstToken};
use sway_core::{
    semantic_analysis::ast_node::{
        expression::typed_expression_variant::TypedExpressionVariant, TypedDeclaration,
    },
    Declaration, Expression, Literal,
};
use tower_lsp::lsp_types::SymbolKind;

pub fn parsed_to_symbol_kind(typed_ast_token: &AstToken) -> SymbolKind {
    match typed_ast_token {
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
                | Declaration::StorageDeclaration(_) => SymbolKind::VARIABLE,
            }
        }
        AstToken::Expression(exp) => {
            match &exp {
                Expression::Literal { value, .. } => match value {
                    Literal::String(_) => SymbolKind::STRING,
                    Literal::Boolean(_) => SymbolKind::BOOLEAN,
                    _ => SymbolKind::NUMBER,
                },
                Expression::FunctionApplication { .. } => SymbolKind::FUNCTION,
                Expression::VariableExpression { .. } => SymbolKind::VARIABLE,
                Expression::Array { .. } => SymbolKind::ARRAY,
                Expression::StructExpression { .. } => SymbolKind::STRUCT,
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

pub fn typed_to_symbol_kind(typed_ast_token: &TypedAstToken) -> SymbolKind {
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
                | TypedDeclaration::StorageReassignment(_) => SymbolKind::VARIABLE,
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
