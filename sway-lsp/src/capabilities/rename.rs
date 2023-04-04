use crate::{
    core::{
        session::Session,
        token::{get_range_from_span, SymbolKind, Token, TypedAstToken},
    },
    error::{LanguageServerError, RenameError},
};
use std::{collections::HashMap, sync::Arc};
use sway_core::{
    language::ty::{TyDecl, TyTraitInterfaceItem, TyTraitItem},
    Engines,
};
use sway_types::{Ident, Spanned};
use tower_lsp::lsp_types::{Position, PrepareRenameResponse, TextEdit, Url, WorkspaceEdit};

const RAW_IDENTIFIER: &str = "r#";

pub fn rename(
    session: Arc<Session>,
    new_name: String,
    url: Url,
    position: Position,
) -> Result<WorkspaceEdit, LanguageServerError> {
    // Make sure the new name is not a keyword
    if sway_parse::RESERVED_KEYWORDS.contains(&new_name) {
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

    //------------------------------------------------------
    eprintln!("INITIAL POSITION: {:#?} \n", position);

    let te = session.type_engine.read();
    let de = session.decl_engine.read();
    let engines = Engines::new(&te, &de);

    // Find the parent declaration
    let (decl_ident, decl_token) = session
        .token_map()
        .parent_decl_at_position(&url, position)
        .ok_or(RenameError::TokenNotFound)?;

    eprintln!("DECL IDENT: {:#?} \n", decl_ident);

    session
        .token_map()
        .all_references_of_token(
            &decl_token,
            &session.type_engine.read(),
            &session.decl_engine.read(),
        )
        .for_each(|(_, token)| {
            if let Some(TypedAstToken::TypedDeclaration(decl)) = &token.typed {
                let method_idents: Vec<_> = match decl {
                    TyDecl::AbiDecl { decl_id, .. } => {
                        let abi_decl = engines.de().get_abi(decl_id);
                        trait_interface_idents(&abi_decl.interface_surface)
                    }
                    TyDecl::TraitDecl { decl_id, .. } => {
                        let trait_decl = engines.de().get_trait(decl_id);
                        trait_interface_idents(&trait_decl.interface_surface)
                    }
                    TyDecl::ImplTrait { decl_id, .. } => {
                        let impl_trait = engines.de().get_impl_trait(decl_id);
                        impl_trait
                            .items
                            .iter()
                            .flat_map(|item| match item {
                                TyTraitItem::Fn(fn_decl) => Some(fn_decl.name().clone()),
                                _ => None,
                            })
                            .collect()
                    }
                    _ => vec![],
                };

                eprintln!("method_idents = {:#?}", method_idents);
            }
        });
    //------------------------------------------------------

    let (_, token) = session
        .token_map()
        .token_at_position(&url, position)
        .ok_or(RenameError::TokenNotFound)?;

    // We don't currently allow renaming of module names.
    if token.kind == SymbolKind::Module {
        return Err(LanguageServerError::RenameError(
            RenameError::UnableToRenameModule { path: new_name },
        ));
    }

    let map_of_changes: HashMap<Url, Vec<TextEdit>> = session
        .token_map()
        .all_references_of_token(
            &token,
            &session.type_engine.read(),
            &session.decl_engine.read(),
        )
        .filter_map(|(ident, _)| {
            let mut range = get_range_from_span(&ident.span());
            if ident.is_raw_ident() {
                // Make sure the start char starts at the begining,
                // taking the r# tokens into account.
                range.start.character -= RAW_IDENTIFIER.len() as u32;
            }
            if let Some(path) = ident.span().path() {
                let url = session.sync.url_from_path(path).ok()?;
                if let Some(url) = session.sync.to_workspace_url(url) {
                    let edit = TextEdit::new(range, new_name.clone());
                    return Some((url, vec![edit]));
                };
            }
            None
        })
        .fold(HashMap::new(), |mut map, (k, mut v)| {
            map.entry(k)
                .and_modify(|existing| {
                    existing.append(&mut v);
                    // Sort the TextEdits by their range in reverse order so the client applies edits
                    // from the end of the document to the beginning, preventing issues with offset changes.
                    existing.sort_unstable_by(|a, b| b.range.start.cmp(&a.range.start))
                })
                .or_insert(v);
            map
        });

    Ok(WorkspaceEdit::new(map_of_changes))
}

pub fn prepare_rename(
    session: Arc<Session>,
    url: Url,
    position: Position,
) -> Result<PrepareRenameResponse, LanguageServerError> {
    let (ident, token) = session
        .token_map()
        .token_at_position(&url, position)
        .ok_or(RenameError::TokenNotFound)?;

    // Only let through tokens that are in the users workspace.
    // tokens that are external to the users workspace cannot be renamed.
    let _ = is_token_in_workspace(&session, &token)?;

    // Make sure we don't allow renaming of tokens that
    // are keywords or intrinsics.
    if matches!(token.kind, SymbolKind::Keyword | SymbolKind::Intrinsic) {
        return Err(LanguageServerError::RenameError(
            RenameError::SymbolKindNotAllowed,
        ));
    }

    Ok(PrepareRenameResponse::RangeWithPlaceholder {
        range: get_range_from_span(&ident.span()),
        placeholder: formatted_name(&ident),
    })
}

/// Returns the name of the identifier, prefixed with r# if the identifier is raw.
fn formatted_name(ident: &Ident) -> String {
    let name = ident.as_str().to_string();
    // Prefix r# onto the name if the ident is raw.
    if ident.is_raw_ident() {
        return format!("{RAW_IDENTIFIER}{name}");
    }
    name
}

/// Checks if the token is in the users workspace.
fn is_token_in_workspace(
    session: &Arc<Session>,
    token: &Token,
) -> Result<bool, LanguageServerError> {
    let decl_span = token
        .declared_token_span(&session.type_engine.read(), &session.decl_engine.read())
        .ok_or(RenameError::TokenNotFound)?;

    // Check the span of the tokens defintions to determine if it's in the users workspace.
    let temp_path = &session.sync.temp_dir()?;
    if let Some(path) = decl_span.path() {
        if !path.starts_with(temp_path) {
            return Err(LanguageServerError::RenameError(
                RenameError::TokenNotPartOfWorkspace,
            ));
        }
    }
    Ok(true)
}

/// Returns a `Vec<Ident>` containing the identifiers of all trait functions found.
fn trait_interface_idents(interface_surface: &[TyTraitInterfaceItem]) -> Vec<Ident> {
    interface_surface
        .iter()
        .flat_map(|item| match item {
            TyTraitInterfaceItem::TraitFn(fn_decl) => Some(fn_decl.name().clone()),
            _ => None,
        })
        .collect()
}
