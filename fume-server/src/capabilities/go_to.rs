use std::sync::Arc;

use crate::core::{session::Session, token::ExpressionType};
use lspower::lsp::{GotoDefinitionParams, GotoDefinitionResponse, Location};

pub fn go_to_definition(
    session: Arc<Session>,
    params: GotoDefinitionParams,
) -> Option<GotoDefinitionResponse> {
    let url = params.text_document_position_params.text_document.uri;

    match session.get_token_at_position(&url, params.text_document_position_params.position) {
        Some(token) => {
            if token.expression_type == ExpressionType::Declaration {
                Some(GotoDefinitionResponse::Scalar(Location::new(
                    url,
                    token.range,
                )))
            } else {
                if let Some(token_definition) = session.get_token_by_name_and_expression_type(
                    &url,
                    &token.name,
                    ExpressionType::Declaration,
                ) {
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
