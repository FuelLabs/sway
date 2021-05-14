use std::sync::Arc;

use crate::core::{
    session::Session,
    token::{Token, TokenType},
};
use lspower::lsp::{GotoDefinitionParams, GotoDefinitionResponse, Location};

pub fn go_to_definition(
    session: Arc<Session>,
    params: GotoDefinitionParams,
) -> Option<GotoDefinitionResponse> {
    let url = params.text_document_position_params.text_document.uri;

    match session.get_token_from_position(&url, params.text_document_position_params.position) {
        Some(token) => {
            if is_definition(&token) {
                Some(GotoDefinitionResponse::Scalar(Location::new(
                    url,
                    token.range,
                )))
            } else {
                if let Some(token_definition) =
                    session.get_token_with_name_and_type(&url, &token.name, &token.token_type)
                {
                    Some(GotoDefinitionResponse::Scalar(Location::new(
                        url,
                        token_definition.range,
                    )))
                } else {
                    None
                }
            }
        }
        _ => None,
    }
}

fn is_definition(token: &Token) -> bool {
    match token.token_type {
        TokenType::FunctionCall => false,
        _ => true,
    }
}
