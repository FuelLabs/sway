use crate::core::{
    session::Session,
    symbol_kind,
    token::{TokenMap, TokenType},
};
use crate::utils::common::get_range_from_span;
use std::sync::Arc;
use sway_types::{Ident, Spanned};
use tower_lsp::lsp_types::{DocumentSymbolResponse, Location, SymbolInformation, Url};

pub fn document_symbol(session: Arc<Session>, url: Url) -> Option<DocumentSymbolResponse> {
    session
        .get_symbol_information(&url)
        .map(DocumentSymbolResponse::Flat)
}

pub fn to_symbol_information(token_map: &TokenMap, url: Url) -> Vec<SymbolInformation> {
    let mut symbols: Vec<SymbolInformation> = vec![];

    for ((ident, _), token) in token_map {
        let symbol = create_symbol_info(ident, token, url.clone());
        symbols.push(symbol)
    }

    symbols
}

#[allow(warnings)]
// TODO: the "deprecated: None" field is deprecated according to this library
fn create_symbol_info(ident: &Ident, token: &TokenType, url: Url) -> SymbolInformation {
    let range = get_range_from_span(&ident.span());
    SymbolInformation {
        name: ident.as_str().to_string(),
        kind: {
            match token.typed {
                Some(typed_token) => symbol_kind::typed_to_symbol_kind(&typed_token),
                None => symbol_kind::parsed_to_symbol_kind(&token.parsed),
            }
        },
        location: Location::new(url, range),
        tags: None,
        container_name: None,
        deprecated: None,
    }
}
