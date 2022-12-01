use crate::core::{
    token::{AstToken, SymbolKind, Token, TypedAstToken},
    token_map::TokenMap,
};
use tower_lsp::lsp_types::{CompletionItem, CompletionItemKind};

pub fn to_completion_items(token_map: &TokenMap) -> Vec<CompletionItem> {
    let mut completion_items = vec![];

    let is_initial_declaration = |token_type: &Token| -> bool {
        match &token_type.typed {
            Some(typed_ast_token) => {
                matches!(
                    typed_ast_token,
                    TypedAstToken::TypedDeclaration(_) | TypedAstToken::TypedFunctionDeclaration(_)
                )
            }
            None => {
                matches!(
                    token_type.parsed,
                    AstToken::Declaration(_) | AstToken::FunctionDeclaration(_)
                )
            }
        }
    };

    for item in token_map.iter() {
        let ((ident, _), token) = item.pair();
        if is_initial_declaration(token) {
            let item = CompletionItem {
                label: ident.as_str().to_string(),
                kind: completion_item_kind(&token.kind),
                ..Default::default()
            };
            completion_items.push(item);
        }
    }

    completion_items
}

/// Given a `SymbolKind`, return the `lsp_types::CompletionItemKind` that corresponds to it.
pub fn completion_item_kind(symbol_kind: &SymbolKind) -> Option<CompletionItemKind> {
    match symbol_kind {
        SymbolKind::Field => Some(CompletionItemKind::FIELD),
        SymbolKind::BuiltinType => Some(CompletionItemKind::TYPE_PARAMETER),
        SymbolKind::ValueParam => Some(CompletionItemKind::VALUE),
        SymbolKind::Function => Some(CompletionItemKind::FUNCTION),
        SymbolKind::Const => Some(CompletionItemKind::CONSTANT),
        SymbolKind::Struct => Some(CompletionItemKind::STRUCT),
        SymbolKind::Trait => Some(CompletionItemKind::INTERFACE),
        SymbolKind::Module => Some(CompletionItemKind::MODULE),
        SymbolKind::Enum => Some(CompletionItemKind::ENUM),
        SymbolKind::Variant => Some(CompletionItemKind::ENUM_MEMBER),
        SymbolKind::TypeParameter => Some(CompletionItemKind::TYPE_PARAMETER),
        SymbolKind::BoolLiteral
        | SymbolKind::ByteLiteral
        | SymbolKind::StringLiteral
        | SymbolKind::NumericLiteral => Some(CompletionItemKind::VALUE),
        SymbolKind::Variable => Some(CompletionItemKind::VARIABLE),
        SymbolKind::Unknown => None,
    }
}
