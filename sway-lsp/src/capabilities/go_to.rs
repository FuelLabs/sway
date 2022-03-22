use std::sync::Arc;

use crate::core::{session::Session, token::Token};
use tower_lsp::lsp_types::{GotoDefinitionParams, GotoDefinitionResponse, Location, Url};

pub fn go_to_definition(
    session: Arc<Session>,
    params: GotoDefinitionParams,
) -> Option<GotoDefinitionResponse> {
    let url = params.text_document_position_params.text_document.uri;
    let position = params.text_document_position_params.position;

    session.get_token_definition_response(url, position)
}

pub fn to_definition_response(url: Url, token: &Token) -> GotoDefinitionResponse {
    GotoDefinitionResponse::Scalar(Location::new(url, token.range))
}
