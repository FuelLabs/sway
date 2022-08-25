use crate::core::token::{SymbolKind, TokenMap};
use crate::utils::token::is_initial_declaration;
use tower_lsp::lsp_types::{CompletionItem, CompletionItemKind};

pub fn to_completion_items(token_map: &TokenMap) -> Vec<CompletionItem> {
    let mut completion_items = vec![];

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

pub(crate) fn completion_item_kind(symbol_kind: &SymbolKind) -> Option<CompletionItemKind> {
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
        SymbolKind::BoolLiteral
        | SymbolKind::ByteLiteral
        | SymbolKind::StringLiteral
        | SymbolKind::NumericLiteral => Some(CompletionItemKind::VALUE),
        SymbolKind::Variable => Some(CompletionItemKind::VARIABLE),
        SymbolKind::Unknown => None,
    }
}
