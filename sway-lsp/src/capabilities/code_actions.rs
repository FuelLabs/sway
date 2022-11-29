pub mod abi_impl;

use crate::core::{session::Session, token::TypedAstToken};
pub use crate::error::DocumentError;
use abi_impl::abi_impl_code_action;
use std::sync::Arc;
use sway_core::{declaration_engine, language::ty::TyDeclaration};
use sway_types::Spanned;
use tower_lsp::lsp_types::{CodeActionResponse, Range, TextDocumentIdentifier, Url};

pub(crate) fn code_actions(
    session: Arc<Session>,
    range: &Range,
    text_document: TextDocumentIdentifier,
    temp_uri: &Url,
) -> Option<CodeActionResponse> {
    let (_, token) = session.token_at_position(temp_uri, range.start.clone())?;
    token.typed.and_then(|typed_token| {
        let maybe_decl = match typed_token {
            TypedAstToken::TypedDeclaration(decl) => Some(decl),
            _ => None,
        };

        maybe_decl
            .and_then(|decl| match decl {
                TyDeclaration::AbiDeclaration(ref decl_id) => Some(declaration_engine::de_get_abi(
                    decl_id.clone(),
                    &decl_id.span(),
                )),
                // Add code actions for other declaration types here
                _ => None,
            })
            .and_then(|result| {
                result.ok().and_then(|abi_decl| {
                    Some(vec![abi_impl_code_action(abi_decl, text_document.uri)])
                })
            })
    })
}

// todo: tests
