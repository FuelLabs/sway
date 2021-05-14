use std::sync::Arc;

use lspower::lsp::{DocumentSymbolResponse, Location, SymbolInformation, SymbolKind, Url};

use crate::core::{
    session::Session,
    token::{Token, TokenType},
};

pub fn document_symbol(session: Arc<Session>, url: Url) -> Option<DocumentSymbolResponse> {
    match session.get_tokens_from_file(&url) {
        Some(tokens) => {
            let mut symbols: Vec<SymbolInformation> = vec![];

            for token in tokens {
                let symbol = create_symbol_info(token, url.clone());
                symbols.push(symbol)
            }

            Some(DocumentSymbolResponse::Flat(symbols))
        }
        _ => None,
    }
}

fn create_symbol_info(token: Token, url: Url) -> SymbolInformation {
    SymbolInformation {
        name: token.name,
        kind: get_kind(&token.token_type),
        location: Location::new(url, token.range),
        tags: None,
        container_name: None,
        deprecated: None,
    }
}

fn get_kind(token_type: &TokenType) -> SymbolKind {
    match token_type {
        TokenType::EnumDefinition => SymbolKind::Enum,
        TokenType::FunctionDefinition => SymbolKind::Function,
        TokenType::LibraryDefinition => SymbolKind::Module,
        TokenType::StructDefinition => SymbolKind::Struct,
        TokenType::VariableDefinition => SymbolKind::Variable,
        TokenType::TraitDefinition => SymbolKind::Interface,
        _ => SymbolKind::Unknown,
    }
}
