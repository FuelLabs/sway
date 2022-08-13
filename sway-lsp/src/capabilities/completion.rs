use crate::core::token::{AstToken, TokenMap, TypedAstToken};
use crate::utils::token::is_initial_declaration;
use sway_core::{
    semantic_analysis::ast_node::{
        expression::typed_expression_variant::TypedExpressionVariant, TypedDeclaration,
    },
    Declaration, ExpressionKind,
};
use tower_lsp::lsp_types::{CompletionItem, CompletionItemKind};

pub fn to_completion_items(token_map: &TokenMap) -> Vec<CompletionItem> {
    let mut completion_items = vec![];

    for item in token_map.iter() {
        let ((ident, _), token) = item.pair();
        if is_initial_declaration(token) {
            let item = CompletionItem {
                label: ident.as_str().to_string(),
                kind: {
                    match &token.typed {
                        Some(typed_token) => typed_to_completion_kind(typed_token),
                        None => parsed_to_completion_kind(&token.parsed),
                    }
                },
                ..Default::default()
            };
            completion_items.push(item);
        }
    }

    completion_items
}

pub fn parsed_to_completion_kind(ast_token: &AstToken) -> Option<CompletionItemKind> {
    match ast_token {
        AstToken::Declaration(dec) => match dec {
            Declaration::VariableDeclaration(_) => Some(CompletionItemKind::VARIABLE),
            Declaration::FunctionDeclaration(_) => Some(CompletionItemKind::FUNCTION),
            Declaration::TraitDeclaration(_) => Some(CompletionItemKind::INTERFACE),
            Declaration::StructDeclaration(_) => Some(CompletionItemKind::STRUCT),
            Declaration::EnumDeclaration(_) => Some(CompletionItemKind::ENUM),
            Declaration::ConstantDeclaration(_) => Some(CompletionItemKind::CONSTANT),
            Declaration::ImplTrait { .. }
            | Declaration::ImplSelf(_)
            | Declaration::AbiDeclaration(_)
            | Declaration::Reassignment(_)
            | Declaration::StorageDeclaration(_) => Some(CompletionItemKind::TEXT),
            Declaration::Break { .. } | Declaration::Continue { .. } => None,
        },
        AstToken::Expression(exp) => match &exp.kind {
            ExpressionKind::Literal { .. } => Some(CompletionItemKind::VALUE),
            ExpressionKind::FunctionApplication { .. } => Some(CompletionItemKind::FUNCTION),
            ExpressionKind::Variable { .. } => Some(CompletionItemKind::VARIABLE),
            ExpressionKind::Struct { .. } => Some(CompletionItemKind::STRUCT),
            _ => None,
        },
        AstToken::FunctionDeclaration(_) => Some(CompletionItemKind::FUNCTION),
        AstToken::FunctionParameter(_) => Some(CompletionItemKind::TYPE_PARAMETER),
        AstToken::StructField(_) => Some(CompletionItemKind::FIELD),
        AstToken::EnumVariant(_) => Some(CompletionItemKind::ENUM_MEMBER),
        AstToken::TraitFn(_) => Some(CompletionItemKind::FUNCTION),
        AstToken::StorageField(_) => Some(CompletionItemKind::FIELD),
        AstToken::Reassignment(_) => Some(CompletionItemKind::VARIABLE),
    }
}

pub fn typed_to_completion_kind(typed_ast_token: &TypedAstToken) -> Option<CompletionItemKind> {
    match typed_ast_token {
        TypedAstToken::TypedDeclaration(dec) => match dec {
            TypedDeclaration::VariableDeclaration(_) => Some(CompletionItemKind::VARIABLE),
            TypedDeclaration::FunctionDeclaration(_) => Some(CompletionItemKind::FUNCTION),
            TypedDeclaration::TraitDeclaration(_) => Some(CompletionItemKind::INTERFACE),
            TypedDeclaration::StructDeclaration(_) => Some(CompletionItemKind::STRUCT),
            TypedDeclaration::EnumDeclaration(_) => Some(CompletionItemKind::ENUM),
            TypedDeclaration::ConstantDeclaration(_) => Some(CompletionItemKind::CONSTANT),
            TypedDeclaration::ImplTrait { .. }
            | TypedDeclaration::AbiDeclaration(_)
            | TypedDeclaration::Reassignment(_)
            | TypedDeclaration::StorageDeclaration(_)
            | TypedDeclaration::StorageReassignment(_) => Some(CompletionItemKind::TEXT),
            _ => None,
        },
        TypedAstToken::TypedExpression(exp) => match &exp.expression {
            TypedExpressionVariant::Literal(_) => Some(CompletionItemKind::VALUE),
            TypedExpressionVariant::FunctionApplication { .. } => {
                Some(CompletionItemKind::FUNCTION)
            }
            TypedExpressionVariant::VariableExpression { .. } => Some(CompletionItemKind::VARIABLE),
            TypedExpressionVariant::StructExpression { .. } => Some(CompletionItemKind::STRUCT),
            _ => None,
        },
        TypedAstToken::TypedFunctionDeclaration(_) => Some(CompletionItemKind::FUNCTION),
        TypedAstToken::TypedFunctionParameter(_) => Some(CompletionItemKind::TYPE_PARAMETER),
        TypedAstToken::TypedStructField(_) => Some(CompletionItemKind::FIELD),
        TypedAstToken::TypedEnumVariant(_) => Some(CompletionItemKind::ENUM_MEMBER),
        TypedAstToken::TypedTraitFn(_) => Some(CompletionItemKind::FUNCTION),
        TypedAstToken::TypedStorageField(_) => Some(CompletionItemKind::FIELD),
        TypedAstToken::TypedReassignment(_) => Some(CompletionItemKind::VARIABLE),
        TypedAstToken::TypeCheckedStorageReassignDescriptor(_) => None,
    }
}
