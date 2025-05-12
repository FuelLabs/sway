use crate::{
    core::{
        session::Session, sync::SyncWorkspace, token::{SymbolKind, Token, TokenIdent, TypedAstToken}, token_map::TokenMapExt
    },
    error::{LanguageServerError, RenameError},
    utils::document::get_url_from_path,
};
use lsp_types::{Position, PrepareRenameResponse, TextEdit, Url, WorkspaceEdit};
use std::{collections::HashMap, sync::Arc};
use sway_core::{language::ty, Engines};
use sway_types::SourceEngine;

const RAW_IDENTIFIER: &str = "r#";

pub fn rename(
    session: Arc<Session>,
    new_name: String,
    url: &Url,
    position: Position,
    sync: &SyncWorkspace,
) -> Result<WorkspaceEdit, LanguageServerError> {
    let _p = tracing::trace_span!("rename").entered();
    // Make sure the new name is not a keyword or a literal int type
    if sway_parse::RESERVED_KEYWORDS.contains(&new_name)
        || sway_parse::parse_int_suffix(&new_name).is_some()
    {
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

    // Get the token at the current cursor position
    let t = session
        .token_map()
        .token_at_position(url, position)
        .ok_or(RenameError::TokenNotFound)?;
    let token = t.value();

    // We don't currently allow renaming of module names.
    if token.kind == SymbolKind::Module {
        return Err(LanguageServerError::RenameError(
            RenameError::UnableToRenameModule { path: new_name },
        ));
    }

    // If the token is a function, find the parent declaration
    // and collect idents for all methods of ABI Decl, Trait Decl, and Impl Trait
    let map_of_changes: HashMap<Url, Vec<TextEdit>> = (if token.kind == SymbolKind::Function {
        find_all_methods_for_decl(&session, &session.engines.read(), url, position)?
    } else {
        // otherwise, just find all references of the token in the token map
        session
            .token_map()
            .iter()
            .all_references_of_token(token, &session.engines.read())
            .map(|item| item.key().clone())
            .collect::<Vec<TokenIdent>>()
    })
    .into_iter()
    .filter_map(|ident| {
        if ident.name == "self" {
            return None;
        }
        let mut range = ident.range;
        if ident.is_raw_ident() {
            // Make sure the start char starts at the beginning,
            // taking the r# tokens into account.
            range.start.character -= RAW_IDENTIFIER.len() as u32;
        }
        if let Some(path) = &ident.path {
            let url = get_url_from_path(path).ok()?;
            if let Some(url) = sync.to_workspace_url(url) {
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
                existing.sort_unstable_by(|a, b| b.range.start.cmp(&a.range.start));
            })
            .or_insert(v);
        map
    });
    Ok(WorkspaceEdit::new(map_of_changes))
}

pub fn prepare_rename(
    session: Arc<Session>,
    url: &Url,
    position: Position,
    sync: &SyncWorkspace,
) -> Result<PrepareRenameResponse, LanguageServerError> {
    let t = session
        .token_map()
        .token_at_position(url, position)
        .ok_or(RenameError::TokenNotFound)?;
    let (ident, token) = t.pair();

    // Only let through tokens that are in the users workspace.
    // tokens that are external to the users workspace cannot be renamed.
    let _ = is_token_in_workspace(&session.engines.read(), token, sync)?;

    // Make sure we don't allow renaming of tokens that
    // are keywords or intrinsics.
    if matches!(
        token.kind,
        SymbolKind::Keyword
            | SymbolKind::SelfKeyword
            | SymbolKind::SelfTypeKeyword
            | SymbolKind::ProgramTypeKeyword
            | SymbolKind::BoolLiteral
            | SymbolKind::Intrinsic
    ) {
        return Err(LanguageServerError::RenameError(
            RenameError::SymbolKindNotAllowed,
        ));
    }

    Ok(PrepareRenameResponse::RangeWithPlaceholder {
        range: ident.range,
        placeholder: formatted_name(ident),
    })
}

/// Returns the name of the identifier, prefixed with r# if the identifier is raw.
fn formatted_name(ident: &TokenIdent) -> String {
    let name = ident.name.to_string();
    // Prefix r# onto the name if the ident is raw.
    if ident.is_raw_ident() {
        return format!("{RAW_IDENTIFIER}{name}");
    }
    name
}

/// Checks if the token is in the users workspace.
fn is_token_in_workspace(
    engines: &Engines,
    token: &Token,
    sync: &SyncWorkspace,
) -> Result<bool, LanguageServerError> {
    let decl_ident = token
        .declared_token_ident(engines)
        .ok_or(RenameError::TokenNotFound)?;

    // Check the span of the tokens definitions to determine if it's in the users workspace.
    let temp_path = &sync.temp_dir()?;
    if let Some(path) = &decl_ident.path {
        if !path.starts_with(temp_path) {
            return Err(LanguageServerError::RenameError(
                RenameError::TokenNotPartOfWorkspace,
            ));
        }
    }

    Ok(true)
}

/// Returns a `Vec<Ident>` containing the identifiers of all trait functions found.
fn trait_interface_idents<'a>(
    interface_surface: &'a [ty::TyTraitInterfaceItem],
    se: &'a SourceEngine,
) -> Vec<TokenIdent> {
    interface_surface
        .iter()
        .filter_map(|item| match item {
            ty::TyTraitInterfaceItem::TraitFn(fn_decl) => Some(TokenIdent::new(fn_decl.name(), se)),
            _ => None,
        })
        .collect()
}

/// Returns the `Ident`s of all methods found for an `AbiDecl`, `TraitDecl`, or `ImplTrait`.
fn find_all_methods_for_decl<'a>(
    session: &'a Session,
    engines: &'a Engines,
    url: &'a Url,
    position: Position,
) -> Result<Vec<TokenIdent>, LanguageServerError> {
    // Find the parent declaration
    let t = session
        .token_map()
        .parent_decl_at_position(engines, url, position)
        .ok_or(RenameError::TokenNotFound)?;
    let decl_token = t.value();

    let idents = session
        .token_map()
        .iter()
        .all_references_of_token(decl_token, engines)
        .filter_map(|item| {
            let token = item.value();
            token.as_typed().as_ref().and_then(|typed| match typed {
                TypedAstToken::TypedDeclaration(decl) => match decl {
                    ty::TyDecl::AbiDecl(ty::AbiDecl { decl_id, .. }) => {
                        let abi_decl = engines.de().get_abi(decl_id);
                        Some(trait_interface_idents(
                            &abi_decl.interface_surface,
                            engines.se(),
                        ))
                    }
                    ty::TyDecl::TraitDecl(ty::TraitDecl { decl_id, .. }) => {
                        let trait_decl = engines.de().get_trait(decl_id);
                        Some(trait_interface_idents(
                            &trait_decl.interface_surface,
                            engines.se(),
                        ))
                    }
                    ty::TyDecl::ImplSelfOrTrait(ty::ImplSelfOrTrait { decl_id, .. }) => {
                        let impl_trait = engines.de().get_impl_self_or_trait(decl_id);
                        Some(
                            impl_trait
                                .items
                                .iter()
                                .filter_map(|item| match item {
                                    ty::TyTraitItem::Fn(fn_decl) => {
                                        Some(TokenIdent::new(fn_decl.name(), engines.se()))
                                    }
                                    _ => None,
                                })
                                .collect::<Vec<TokenIdent>>(),
                        )
                    }
                    _ => None,
                },
                _ => None,
            })
        })
        .flatten()
        .collect();
    Ok(idents)
}
