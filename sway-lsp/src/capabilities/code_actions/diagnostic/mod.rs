use std::{collections::HashMap, ops::Deref};

use crate::{
    capabilities::{
        code_actions::{diagnostic, CodeAction, CodeActionContext},
        diagnostic::DiagnosticData,
    },
    core::token::{TypeDefinition, TypedAstToken},
};
use lsp_types::{
    CodeAction as LspCodeAction, CodeActionKind, CodeActionOrCommand, Range, TextEdit,
    WorkspaceEdit,
};
use serde_json::Value;
use sway_core::{
    language::{
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
    // TODO: check for diagnostics
    if let Some(namespace) = namespace {
        Some(vec![import_code_action(ctx, namespace)])
    } else {
        None
    }
}

/// Returns a [CodeActionOrCommand] for the given code action.
fn import_code_action(ctx: &CodeActionContext, namespace: &Namespace) -> CodeActionOrCommand {
    let diag_data = ctx
        .diagnostics
        .iter()
        .find_map(|diag| serde_json::from_value::<DiagnosticData>(diag.clone().data.unwrap()).ok())
        .unwrap();

    eprintln!("diag_data: {:?}", diag_data);

    // Check if there is a type to import using the name from the diagnostic data.
    let call_paths: Vec<CallPath> = ctx
        .tokens
        .tokens_for_name(&diag_data.name_to_import)
        .filter_map(|(_, token)| {
            // If the typed token is a declaration, then we can import it.
            if let Some(TypedAstToken::TypedDeclaration(ty_decl)) = token.typed.as_ref() {
                // match token.type_def.as_ref() {
                //     Some(TypeDefinition::Ident(_)) =>
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
                    TyDecl::ConstantDecl(decl) => {
                        let constant_decl = ctx.engines.de().get_constant(&decl.decl_id);
                        let call_path = constant_decl.call_path.to_import_path(&namespace);
                        Some(call_path)
                    }
                    // TyDecl::TraitDecl(decl) => {
                    //     let trait_decl = ctx.engines.de().get_trait(&decl.decl_id);
                    //     let call_path = trait_decl.call_path.to_import_path(&namespace);
                    //     Some(call_path)
                    // }
                    // TyDecl::FunctionDecl(decl) => {
                    //     let function_decl = ctx.engines.de().get_function(&decl.decl_id);
                    //     let call_path = function_decl.call_path.to_import_path(&namespace);
                    //     Some(call_path)
                    // }
                    // TyDecl::TypeAliasDecl(decl) => {
                    //     let type_alias_decl = ctx.engines.de().get_type_alias(&decl.decl_id);
                    //     let call_path = type_alias_decl.call_path.to_import_path(&namespace);
                    //     Some(call_path)
                    // }
                    _ => None, // TODO: other types
                };
                //     _ => None,
                // }
            }
            None
        })
        .collect();

    let text_edit = TextEdit {
        range: Range::default(), // TODO: sort within the imports
        new_text: format!("use {};\n", call_paths[0]), // TODO: multiple imports
    };
    let changes = HashMap::from([(ctx.uri.clone(), vec![text_edit])]);

    CodeActionOrCommand::CodeAction(LspCodeAction {
        title: format!("{} {}", CODE_ACTION_IMPORT_TITLE, call_paths[0]),
        kind: Some(CodeActionKind::QUICKFIX),
        edit: Some(WorkspaceEdit {
            changes: Some(changes),
            ..Default::default()
        }),
        data: Some(Value::String(ctx.uri.to_string())),
        ..Default::default()
    })
}
