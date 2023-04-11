use crate::core::{session::Session, token::get_range_from_span};
use std::collections::HashMap;
use std::sync::Arc;
use sway_core::Engines;
use sway_types::Spanned;
use tower_lsp::lsp_types::{Position, PrepareRenameResponse, TextEdit, Url, WorkspaceEdit};

pub fn rename(
    session: Arc<Session>,
    new_name: String,
    url: Url,
    position: Position,
) -> Option<WorkspaceEdit> {
    let (_, token) = session.token_map().token_at_position(&url, position)?;
    let mut edits = Vec::new();

    // todo: currently only supports single file rename
    let te = session.type_engine.read();
    let de = session.decl_engine.read();
    let engines = Engines::new(&te, &de);
    for (ident, _) in session.token_map().all_references_of_token(&token, engines) {
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
    let (ident, ..) = session.token_map().token_at_position(&url, position)?;
    Some(PrepareRenameResponse::RangeWithPlaceholder {
        range: get_range_from_span(&ident.span()),
        placeholder: ident.as_str().to_string(),
    })
}
