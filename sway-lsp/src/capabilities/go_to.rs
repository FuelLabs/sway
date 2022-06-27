use std::sync::Arc;

use crate::{core::session::Session, utils::common::get_range_from_span};
use sway_types::{Ident, Spanned};
use tower_lsp::lsp_types::{GotoDefinitionParams, GotoDefinitionResponse, Location, Url};

pub fn go_to_definition(
    session: Arc<Session>,
    params: GotoDefinitionParams,
) -> Option<GotoDefinitionResponse> {
    let url = params.text_document_position_params.text_document.uri;
    let position = params.text_document_position_params.position;

    session.get_token_definition_response(url, position)
}

/// Pass in the Ident that represents the original definition location
pub fn to_definition_response(url: Url, ident: &Ident) -> GotoDefinitionResponse {
    let range = get_range_from_span(&ident.span());
    GotoDefinitionResponse::Scalar(Location::new(url, range))
}
