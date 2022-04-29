use crate::core::{session::Session, token::Token, token_type::TokenType};
use std::sync::Arc;
use tower_lsp::lsp_types::{DocumentSymbolResponse, Location, SymbolInformation, SymbolKind, Url};

pub fn document_symbol(session: Arc<Session>, url: Url) -> Option<DocumentSymbolResponse> {
    session
        .get_symbol_information(&url)
        .map(DocumentSymbolResponse::Flat)
}

pub fn to_symbol_information(tokens: &[Token], url: Url) -> Vec<SymbolInformation> {
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
        TokenType::VariableDeclaration(_) | TokenType::VariableExpression => SymbolKind::VARIABLE,
        TokenType::FunctionDeclaration(_)
        | TokenType::FunctionApplication
        | TokenType::TraitFunction => SymbolKind::FUNCTION,
        TokenType::TraitDeclaration(_) | TokenType::ImplTrait => SymbolKind::INTERFACE,
        TokenType::StructDeclaration(_) | TokenType::Struct => SymbolKind::STRUCT,
        TokenType::EnumDeclaration(_) | TokenType::EnumApplication => SymbolKind::ENUM,
        TokenType::ConstantDeclaration(_) => SymbolKind::CONSTANT,
        TokenType::Library => SymbolKind::MODULE,
        TokenType::Reassignment => SymbolKind::OPERATOR,
        // currently we return `variable` type as default
        _ => SymbolKind::VARIABLE,
    }
}
