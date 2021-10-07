use crate::core::{session::Session, token::Token, token_type::TokenType};
use lspower::lsp::{DocumentSymbolResponse, Location, SymbolInformation, SymbolKind, Url};
use std::sync::Arc;

pub fn document_symbol(session: Arc<Session>, url: Url) -> Option<DocumentSymbolResponse> {
    match session.get_symbol_information(&url) {
        Some(symbols) => Some(DocumentSymbolResponse::Flat(symbols)),
        _ => None,
    }
}

pub fn to_symbol_information(tokens: &Vec<Token>, url: Url) -> Vec<SymbolInformation> {
    let mut symbols: Vec<SymbolInformation> = vec![];

    for token in tokens {
        let symbol = create_symbol_info(token, url.clone());
        symbols.push(symbol)
    }

    symbols
}

#[allow(warnings)]
// TODO: the "deprecated: None" field is deprecated according to this library
fn create_symbol_info(token: &Token, url: Url) -> SymbolInformation {
    SymbolInformation {
        name: token.name.clone(),
        kind: get_kind(&token.token_type),
        location: Location::new(url, token.range),
        tags: None,
        container_name: None,
        deprecated: None,
    }
}

fn get_kind(token_type: &TokenType) -> SymbolKind {
    match token_type {
        TokenType::Enum => SymbolKind::Enum,
        TokenType::FunctionDeclaration(_) | &TokenType::FunctionApplication => SymbolKind::Function,
        TokenType::Library => SymbolKind::Module,
        TokenType::Struct(_) => SymbolKind::Struct,
        TokenType::Variable => SymbolKind::Variable,
        TokenType::Trait(_) => SymbolKind::Interface,
        _ => SymbolKind::Unknown,
    }
}
