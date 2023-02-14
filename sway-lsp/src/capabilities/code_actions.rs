pub mod abi_impl;

use crate::core::{session::Session, token::TypedAstToken};
pub use crate::error::DocumentError;
use abi_impl::abi_impl_code_action;
use std::sync::Arc;
use sway_core::{language::ty::TyDeclaration, Engines};
use tower_lsp::lsp_types::{CodeActionResponse, Range, TextDocumentIdentifier, Url};

pub(crate) fn code_actions(
    session: Arc<Session>,
    range: &Range,
    text_document: TextDocumentIdentifier,
    temp_uri: &Url,
) -> Option<CodeActionResponse> {
    let (_, token) = session
        .token_map()
        .token_at_position(temp_uri, range.start)?;
    token.typed.and_then(|typed_token| {
        let maybe_decl = match typed_token {
            TypedAstToken::TypedDeclaration(decl) => Some(decl),
            _ => None,
        };

        maybe_decl
            .and_then(|decl| match decl {
                TyDeclaration::AbiDeclaration {
                    decl_id, decl_span, ..
                } => Some(session.decl_engine.read().get_abi(&decl_id, &decl_span)),
                // Add code actions for other declaration types here
                _ => None,
            })
            .and_then(|result| {
                result.ok().map(|abi_decl| {
                    vec![abi_impl_code_action(
                        Engines::new(&session.type_engine.read(), &session.decl_engine.read()),
                        abi_decl,
                        text_document.uri,
                    )]
                })
            })
    })
}
