use crate::core::token::{SymbolKind, Token, TokenIdent};
use lsp_types::{self, Location, SymbolInformation, Url};

pub fn to_symbol_information<I>(tokens: I, url: Url) -> Vec<SymbolInformation>
where
    I: Iterator<Item = (TokenIdent, Token)>,
{
    let mut symbols: Vec<SymbolInformation> = vec![];

    for (ident, token) in tokens {
        let symbol = symbol_info(&ident, &token, url.clone());
        symbols.push(symbol)
    }

    symbols
}

/// Given a `token::SymbolKind`, return the `lsp_types::SymbolKind` that corresponds to it.
pub(crate) fn symbol_kind(symbol_kind: &SymbolKind) -> lsp_types::SymbolKind {
    match symbol_kind {
        SymbolKind::Field => lsp_types::SymbolKind::FIELD,
        SymbolKind::BuiltinType => lsp_types::SymbolKind::TYPE_PARAMETER,
        SymbolKind::Function | SymbolKind::DeriveHelper | SymbolKind::Intrinsic => {
            lsp_types::SymbolKind::FUNCTION
        }
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
        | SymbolKind::TypeAlias
        | SymbolKind::TraiType
        | SymbolKind::Keyword
        | SymbolKind::SelfKeyword
        | SymbolKind::SelfTypeKeyword
        | SymbolKind::Unknown => lsp_types::SymbolKind::VARIABLE,
    }
}

#[allow(warnings)]
// TODO: the "deprecated: None" field is deprecated according to this library
fn symbol_info(ident: &TokenIdent, token: &Token, url: Url) -> SymbolInformation {
    SymbolInformation {
        name: ident.name.to_string(),
        kind: symbol_kind(&token.kind),
        location: Location::new(url, ident.range),
        tags: None,
        container_name: None,
        deprecated: None,
    }
}
