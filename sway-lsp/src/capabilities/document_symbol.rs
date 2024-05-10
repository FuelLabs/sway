use crate::core::token::{SymbolKind, Token, TokenIdent};
use dashmap::mapref::multiple::RefMulti;
use lsp_types::{self, Location, SymbolInformation, Url};

pub fn to_symbol_information<'a, I>(tokens: I, url: &Url) -> Vec<SymbolInformation>
where
    I: Iterator<Item = RefMulti<'a, TokenIdent, Token>>,
{
    tokens
        .map(|entry| {
            let (ident, token) = entry.pair();
            symbol_info(ident, token, url.clone())
        })
        .collect()
}

/// Given a `token::SymbolKind`, return the `lsp_types::SymbolKind` that corresponds to it.
pub(crate) fn symbol_kind(symbol_kind: &SymbolKind) -> lsp_types::SymbolKind {
    match symbol_kind {
        SymbolKind::Field => lsp_types::SymbolKind::FIELD,
        SymbolKind::BuiltinType | SymbolKind::TypeParameter => {
            lsp_types::SymbolKind::TYPE_PARAMETER
        }
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
        SymbolKind::ValueParam
        | SymbolKind::ByteLiteral
        | SymbolKind::Variable
        | SymbolKind::TypeAlias
        | SymbolKind::TraitType
        | SymbolKind::Keyword
        | SymbolKind::SelfKeyword
        | SymbolKind::SelfTypeKeyword
        | SymbolKind::ProgramTypeKeyword
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
