use std::sync::Arc;

use lspower::lsp::{DocumentSymbolResponse, Location, SymbolInformation, SymbolKind, Url};

use crate::core::{
    session::Session,
    token::{ContentType, DeclarationType, Token},
};

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

fn create_symbol_info(token: &Token, url: Url) -> SymbolInformation {
    SymbolInformation {
        name: token.name.clone(),
        kind: get_kind(&token.content_type),
        location: Location::new(url, token.range),
        tags: None,
        container_name: None,
        deprecated: None,
    }
}

fn get_kind(content_type: &ContentType) -> SymbolKind {
    if let ContentType::Declaration(dec) = content_type {
        match dec {
            DeclarationType::Enum => SymbolKind::Enum,
            DeclarationType::Function => SymbolKind::Function,
            DeclarationType::Library => SymbolKind::Module,
            DeclarationType::Struct => SymbolKind::Struct,
            DeclarationType::Variable => SymbolKind::Variable,
            DeclarationType::Trait => SymbolKind::Interface,
            _ => SymbolKind::Unknown,
        }
    } else {
        SymbolKind::Unknown
    }
}
