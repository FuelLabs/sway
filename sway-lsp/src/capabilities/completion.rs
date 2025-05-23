use crate::core::token::TokenIdent;
use lsp_types::{
    CompletionItem, CompletionItemKind, CompletionItemLabelDetails, CompletionTextEdit, Position,
    Range, TextEdit,
};
use sway_core::{
    language::ty::{TyAstNodeContent, TyDecl, TyFunctionDecl, TyFunctionParameter},
    Engines, Namespace, TypeId, TypeInfo,
};
use sway_types::Spanned;

pub(crate) fn to_completion_items(
    namespace: &Namespace,
    engines: &Engines,
    ident_to_complete: &TokenIdent,
    fn_decl: &TyFunctionDecl,
    position: Position,
) -> Vec<CompletionItem> {
    type_id_of_raw_ident(engines, namespace, &ident_to_complete.name, fn_decl)
        .map(|type_id| completion_items_for_type_id(engines, namespace, type_id, position))
        .unwrap_or_default()
}

/// Gathers the given [`TypeId`] struct's fields and methods and builds completion items.
fn completion_items_for_type_id(
    engines: &Engines,
    namespace: &Namespace,
    type_id: TypeId,
    position: Position,
) -> Vec<CompletionItem> {
    let mut completion_items = vec![];
    let type_info = engines.te().get(type_id);
    if let TypeInfo::Struct(decl_id) = &*type_info {
        let struct_decl = engines.de().get_struct(&decl_id.clone());
        for field in &struct_decl.fields {
            let item = CompletionItem {
                kind: Some(CompletionItemKind::FIELD),
                label: field.name.as_str().to_string(),
                label_details: Some(CompletionItemLabelDetails {
                    description: Some(field.type_argument.span().clone().str()),
                    detail: None,
                }),
                ..Default::default()
            };
            completion_items.push(item);
        }
    }

    for method in namespace
        .current_module()
        .get_methods_for_type(engines, type_id)
    {
        let method = method.expect_typed();
        let fn_decl = engines.de().get_function(&method.id().clone());
        let params = &fn_decl.parameters;

        // Only show methods that take `self` as the first parameter.
        if params.first().is_some_and(TyFunctionParameter::is_self) {
            let params_short = if params.is_empty() {
                "()".to_string()
            } else {
                "(…)".to_string()
            };
            let params_edit_str = params
                .iter()
                .filter_map(|p| {
                    if p.is_self() {
                        return None;
                    }
                    Some(p.name.as_str())
                })
                .collect::<Vec<&str>>()
                .join(", ");
            let item = CompletionItem {
                kind: Some(CompletionItemKind::METHOD),
                label: format!("{}{}", method.name().clone().as_str(), params_short),
                text_edit: Some(CompletionTextEdit::Edit(TextEdit {
                    range: Range {
                        start: position,
                        end: position,
                    },
                    new_text: format!("{}({})", method.name().clone().as_str(), params_edit_str),
                })),
                label_details: Some(CompletionItemLabelDetails {
                    description: Some(fn_signature_string(engines, &fn_decl, &type_id)),
                    detail: None,
                }),
                ..Default::default()
            };
            completion_items.push(item);
        }
    }

    completion_items
}

/// Returns the [String] of the shortened function signature to display in the completion item's label details.
fn fn_signature_string(
    engines: &Engines,
    fn_decl: &TyFunctionDecl,
    parent_type_id: &TypeId,
) -> String {
    let params_str = fn_decl
        .parameters
        .iter()
        .map(|p| {
            replace_self_with_type_str(
                engines,
                p.type_argument.clone().span().str(),
                parent_type_id,
            )
        })
        .collect::<Vec<String>>()
        .join(", ");
    format!(
        "fn({}) -> {}",
        params_str,
        replace_self_with_type_str(
            engines,
            fn_decl.return_type.clone().span().str(),
            parent_type_id
        )
    )
}

/// Given a [String] representing a type, replaces `Self` with the display name of the type.
fn replace_self_with_type_str(
    engines: &Engines,
    type_str: String,
    parent_type_id: &TypeId,
) -> String {
    if type_str == "Self" {
        return engines.help_out(parent_type_id).to_string();
    }
    type_str
}

/// Returns the [TypeId] of an ident that may include field accesses and may be incomplete.
/// For the first part of the ident, it looks for instantiation in the scope of the given
/// [`TyFunctionDecl`]. For example, given `a.b.c`, it will return the type ID of `c`
/// if it can resolve `a` in the given function.
fn type_id_of_raw_ident(
    engines: &Engines,
    namespace: &Namespace,
    ident_name: &str,
    fn_decl: &TyFunctionDecl,
) -> Option<TypeId> {
    // If this ident has no field accesses or chained methods, look for it in the local function scope.
    if !ident_name.contains('.') {
        return type_id_of_local_ident(ident_name, fn_decl);
    }

    // Otherwise, start with the first part of the ident and follow the subsequent types.
    let parts = ident_name.split('.').collect::<Vec<&str>>();
    let mut curr_type_id = type_id_of_local_ident(parts[0], fn_decl);
    let mut i = 1;

    while (i < parts.len()) && curr_type_id.is_some() {
        if parts[i].ends_with(')') {
            let method_name = parts[i].split_at(parts[i].find('(').unwrap_or(0)).0;
            curr_type_id = namespace
                .current_module()
                .get_methods_for_type(engines, curr_type_id?)
                .into_iter()
                .find_map(|method| {
                    let method = method.expect_typed();
                    if method.name().clone().as_str() == method_name {
                        return Some(
                            engines
                                .de()
                                .get_function(&method.id().clone())
                                .return_type
                                .type_id(),
                        );
                    }
                    None
                });
        } else if let TypeInfo::Struct(decl_id) = &*engines.te().get(curr_type_id.unwrap()) {
            let struct_decl = engines.de().get_struct(&decl_id.clone());
            curr_type_id = struct_decl
                .fields
                .iter()
                .find(|field| field.name.as_str() == parts[i])
                .map(|field| field.type_argument.type_id());
        }
        i += 1;
    }
    curr_type_id
}

/// Returns the [TypeId] of an ident by looking for its instantiation within the scope of the
/// given [TyFunctionDecl].
fn type_id_of_local_ident(ident_name: &str, fn_decl: &TyFunctionDecl) -> Option<TypeId> {
    fn_decl
        .parameters
        .iter()
        .find_map(|param| {
            // Check if this ident is a function parameter
            if param.name.as_str() == ident_name {
                return Some(param.type_argument.type_id());
            }
            None
        })
        .or_else(|| {
            // Check if there is a variable declaration for this ident
            fn_decl.body.contents.iter().find_map(|node| {
                if let TyAstNodeContent::Declaration(TyDecl::VariableDecl(variable_decl)) =
                    node.content.clone()
                {
                    if variable_decl.name.as_str() == ident_name {
                        return Some(variable_decl.return_type);
                    }
                }
                None
            })
        })
}
