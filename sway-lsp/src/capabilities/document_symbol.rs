use crate::core::token::{SymbolKind, Token, TokenMap};
use crate::utils::common::get_range_from_span;
use sway_types::{Ident, Spanned};
use tower_lsp::lsp_types::{self, Location, SymbolInformation, Url};

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
        kind: symbol_kind(&token.kind),
        location: Location::new(url, range),
        tags: None,
        container_name: None,
        deprecated: None,
    }
}

/// Given a `token::SymbolKind`, return the `lsp_types::SymbolKind` that corresponds to it.
pub(crate) fn symbol_kind(symbol_kind: &SymbolKind) -> lsp_types::SymbolKind {
    match symbol_kind {
        SymbolKind::Field => lsp_types::SymbolKind::FIELD,
        SymbolKind::BuiltinType => lsp_types::SymbolKind::TYPE_PARAMETER,
        SymbolKind::Function => lsp_types::SymbolKind::FUNCTION,
        SymbolKind::Const => lsp_types::SymbolKind::CONSTANT,
        SymbolKind::Struct => lsp_types::SymbolKind::STRUCT,
        SymbolKind::Trait => lsp_types::SymbolKind::INTERFACE,
        SymbolKind::Module => lsp_types::SymbolKind::MODULE,
        SymbolKind::Enum => lsp_types::SymbolKind::ENUM,
        SymbolKind::Variant => lsp_types::SymbolKind::ENUM_MEMBER,
        SymbolKind::BoolLiteral => lsp_types::SymbolKind::BOOLEAN,
        SymbolKind::StringLiteral => lsp_types::SymbolKind::STRING,
        SymbolKind::NumericLiteral => lsp_types::SymbolKind::NUMBER,
        SymbolKind::TypeParameter => lsp_types::SymbolKind::TYPE_PARAMETER,
        SymbolKind::ValueParam
        | SymbolKind::ByteLiteral
        | SymbolKind::Variable
        | SymbolKind::Unknown => lsp_types::SymbolKind::VARIABLE,
    }
}
