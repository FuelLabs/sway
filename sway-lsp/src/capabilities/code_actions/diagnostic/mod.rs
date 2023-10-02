use std::{collections::HashMap, ops::Deref};

use crate::{
    capabilities::{
        code_actions::{diagnostic, CodeAction, CodeActionContext},
        diagnostic::DiagnosticData,
    },
    core::token::{AstToken, TypeDefinition, TypedAstToken},
};
use lsp_types::{
    CodeAction as LspCodeAction, CodeActionKind, CodeActionOrCommand, Range, TextEdit,
    WorkspaceEdit,
};
use serde_json::Value;
use sway_core::{
    language::{
        parsed::Declaration,
        ty::{self, ConstantDecl, TyDecl},
        CallPath,
    },
    namespace, Namespace,
};
use sway_types::Span;

use super::CODE_ACTION_IMPORT_TITLE;

pub(crate) fn code_actions(
    ctx: &CodeActionContext,
    namespace: &Option<Namespace>,
) -> Option<Vec<CodeActionOrCommand>> {
    if let Some(namespace) = namespace {
        return import_code_action(ctx, namespace);
    }
    None
}

/// Returns a [CodeActionOrCommand] for the given code action.
fn import_code_action(
    ctx: &CodeActionContext,
    namespace: &Namespace,
) -> Option<Vec<CodeActionOrCommand>> {
    if let Some(diag_data) = ctx.diagnostics.iter().find_map(|diag| {
        let data = diag.clone().data?;
        serde_json::from_value::<DiagnosticData>(data).ok()
    }) {
        // Check if there is a type to import using the name from the diagnostic data.
        let call_paths = ctx
            .tokens
            .tokens_for_name(&diag_data.name_to_import)
            .filter_map(|(ident, token)| {
                eprintln!("ident: {:?}, ", ident);
                eprintln!("token: {:?}", token);

                // If the typed token is a declaration, then we can import it.
                match token.typed.as_ref() {
                    Some(TypedAstToken::TypedDeclaration(ty_decl)) => {
                        return match ty_decl {
                            TyDecl::StructDecl(decl) => {
                                let struct_decl = ctx.engines.de().get_struct(&decl.decl_id);
                                let call_path = struct_decl.call_path.to_import_path(&namespace);
                                Some(call_path)
                            }
                            TyDecl::EnumDecl(decl) => {
                                let enum_decl = ctx.engines.de().get_enum(&decl.decl_id);
                                let call_path = enum_decl.call_path.to_import_path(&namespace);
                                Some(call_path)
                            }
                            // TyDecl::ConstantDecl(decl) => {
                            //     let constant_decl = ctx.engines.de().get_constant(&decl.decl_id);
                            //     let call_path = constant_decl.call_path.to_import_path(&namespace);
                            //     Some(call_path)
                            // }
                            // TyDecl::TraitDecl(decl) => {
                            //     let trait_decl = ctx.engines.de().get_trait(&decl.decl_id);
                            //     let call_path = trait_decl.call_path.to_import_path(&namespace);
                            //     Some(call_path)
                            // }
                            // TyDecl::TypeAliasDecl(decl) => {
                            //     let type_alias_decl =
                            //         ctx.engines.de().get_type_alias(&decl.decl_id);
                            //     let call_path =
                            //         type_alias_decl.call_path.to_import_path(&namespace);
                            //     Some(call_path)
                            // }
                            _ => None, // TODO: other types
                        };
                    }
                    Some(TypedAstToken::TypedFunctionDeclaration(ty_decl)) => {
                        let call_path = ty_decl.call_path.to_import_path(&namespace);
                        Some(call_path)
                    }
                    Some(TypedAstToken::TypedConstantDeclaration(ty_decl)) => {
                        let call_path = ty_decl.call_path.to_import_path(&namespace);
                        eprintln!("constant call_path: {:?}", call_path);
                        Some(call_path)
                    }
                    _ => return None,
                }
            });

        let actions = call_paths
            .filter_map(|call_path| {
                let text_edit = TextEdit {
                    range: Range::default(), // TODO: sort within the imports
                    new_text: format!("use {};\n", call_path),
                };
                let changes = HashMap::from([(ctx.uri.clone(), vec![text_edit])]);

                return Some(CodeActionOrCommand::CodeAction(LspCodeAction {
                    title: format!("{} `{}`", CODE_ACTION_IMPORT_TITLE, call_path),
                    kind: Some(CodeActionKind::QUICKFIX),
                    edit: Some(WorkspaceEdit {
                        changes: Some(changes),
                        ..Default::default()
                    }),
                    data: Some(Value::String(ctx.uri.to_string())),
                    ..Default::default()
                }));
            })
            .collect::<Vec<_>>();

        if !actions.is_empty() {
            return Some(actions);
        }
    }
    None
}
