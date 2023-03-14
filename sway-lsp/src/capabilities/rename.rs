use crate::{
    core::{
        session::Session,
        token::{get_range_from_span, SymbolKind},
    },
    error::{LanguageServerError, RenameError},
};
use std::collections::HashMap;
use std::sync::Arc;
use sway_core::Engines;
use sway_types::Spanned;
use tower_lsp::lsp_types::{Position, PrepareRenameResponse, TextEdit, Url, WorkspaceEdit};

const RAW_IDENTIFIER: &str = "r#";

pub fn rename(
    session: Arc<Session>,
    new_name: String,
    url: Url,
    position: Position,
) -> Result<WorkspaceEdit, LanguageServerError> {
    // Make sure the new name is not a keyword
    let compiler_keywords: Vec<_> = sway_parse::RESERVED_KEYWORDS
        .iter()
        .map(|s| s.to_string())
        .collect();
    if compiler_keywords.contains(&new_name) {
        return Err(LanguageServerError::RenameError(RenameError::InvalidName {
            name: new_name,
        }));
    }
    // Identifiers cannot begin with a double underscore, this is reserved for compiler intrinsics.
    if new_name.starts_with("__") {
        return Err(LanguageServerError::RenameError(
            RenameError::InvalidDoubleUnderscore,
        ));
    }

    let (ident, token) = session
        .token_map()
        .token_at_position(&url, position)
        .ok_or_else(|| RenameError::TokenNotFound)?;
    let mut map_of_changes: HashMap<Url, Vec<TextEdit>> = HashMap::new();
    for (ident, _) in session.token_map().all_references_of_token(
        &token,
        &session.type_engine.read(),
        &session.decl_engine.read(),
    ) {
        let mut range = get_range_from_span(&ident.span());
        if ident.is_raw_ident() {
            // Make sure the start char starts at the begining,
            // taking the r# tokens into account.
            range.start.character -= RAW_IDENTIFIER.len() as u32;
        }
        if let Some(path) = ident.span().path() {
            let url = session.sync.url_from_path(&path)?;
            session.sync.to_workspace_url(url).map(|url| {
                let edit = TextEdit::new(range, new_name.clone());
                match map_of_changes.get_mut(&url) {
                    Some(edits) => {
                        edits.push(edit);
                    }
                    None => {
                        map_of_changes.insert(url, vec![edit]);
                    }
                }
            });
        }
    }

    Ok(WorkspaceEdit::new(map_of_changes))
}

pub fn prepare_rename(
    session: Arc<Session>,
    url: Url,
    position: Position,
) -> Result<PrepareRenameResponse, LanguageServerError> {
    let temp_path = &session.sync.temp_dir()?;
    let (ident, token) = session
        .token_map()
        .token_at_position(&url, position)
        .ok_or_else(|| RenameError::TokenNotFound)?;

    // Only let through tokens that are in the users workspace.
    // tokens that are external to the users workspace cannot be renamed.
    let decl_span = token
        .declared_token_span(&session.type_engine.read(), &session.decl_engine.read())
        .ok_or_else(|| RenameError::TokenNotFound)?;

    // Check the span of the tokens defintions to determine if it's in the users workspace.
    if let Some(path) = decl_span.path() {
        if !path.starts_with(temp_path) {
            return Err(LanguageServerError::RenameError(
                RenameError::TokenNotPartOfWorkspace,
            ));
        }
    }

    // Make sure we don't allow renaming of tokens that
    // are keywords or intrinsics.
    if matches!(token.kind, SymbolKind::Keyword | SymbolKind::Intrinsic) {
        return Err(LanguageServerError::RenameError(
            RenameError::UnableToRenameKeyword,
        ));
    }

    let mut name = ident.as_str().to_string();
    // Prefix r# onto the name if the ident is raw.
    if ident.is_raw_ident() {
        name = format!("{}{}", RAW_IDENTIFIER, name);
    }

    Ok(PrepareRenameResponse::RangeWithPlaceholder {
        range: get_range_from_span(&ident.span()),
        placeholder: name,
    })
}
