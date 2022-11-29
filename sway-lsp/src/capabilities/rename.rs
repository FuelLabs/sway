use std::collections::HashMap;
use std::sync::Arc;
use sway_types::Spanned;
use tower_lsp::lsp_types::{Position, PrepareRenameResponse, TextEdit, Url, WorkspaceEdit};

use crate::core::{session::Session, token::AstToken};
use crate::utils::token::get_range_from_span;

pub fn rename(
    session: Arc<Session>,
    new_name: String,
    url: Url,
    position: Position,
) -> Option<WorkspaceEdit> {
    let (_, token) = session.token_map().token_at_position(&url, position)?;
    let mut edits = Vec::new();

    // todo: currently only supports single file rename
    for (ident, _) in session
        .token_map()
        .all_references_of_token(&token, &session.type_engine.read())
    {
        let range = get_range_from_span(&ident.span());
        edits.push(TextEdit::new(range, new_name.clone()));
    }

    let mut map_of_changes = HashMap::new();
    session.sync.to_workspace_url(url).map(|url| {
        map_of_changes.insert(url, edits);
        WorkspaceEdit::new(map_of_changes)
    })
}

pub fn prepare_rename(
    session: Arc<Session>,
    url: Url,
    position: Position,
) -> Option<PrepareRenameResponse> {
    let (ident, token) = session.token_map().token_at_position(&url, position)?;
    match token.parsed {
        AstToken::Reassignment(_) => None,
        _ => Some(PrepareRenameResponse::RangeWithPlaceholder {
            range: get_range_from_span(&ident.span()),
            placeholder: ident.as_str().to_string(),
        }),
    }
}
