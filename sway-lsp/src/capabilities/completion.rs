use sway_core::{
    decl_engine::DeclRef,
    language::ty::{TyAstNodeContent, TyDeclaration, TyFunctionDeclaration, TyFunctionParameter},
    namespace::Items,
    Engines, TypeId, TypeInfo,
};
use sway_types::Ident;
use tower_lsp::lsp_types::{
    CompletionItem, CompletionItemKind, CompletionItemLabelDetails, CompletionTextEdit, Position,
    Range, TextEdit,
};

pub(crate) fn to_completion_items(
    namespace: &Items,
    engines: Engines<'_>,
    ident_to_complete: &Ident,
    fn_decl: &TyFunctionDeclaration,
    position: Position,
) -> Vec<CompletionItem> {
    type_id_of_raw_ident(engines, namespace, ident_to_complete, fn_decl)
        .and_then(|type_id| {
            Some(completion_items_for_type_id(
                engines, namespace, type_id, position,
            ))
        })
        .unwrap_or_default()
}

/// Gathers the given [TypeId] struct's fields and methods and builds completion items.
fn completion_items_for_type_id(
    engines: Engines<'_>,
    namespace: &Items,
    type_id: TypeId,
    position: Position,
) -> Vec<CompletionItem> {
    let mut completion_items = vec![];
    let type_info = engines.te().get(type_id);

    if let TypeInfo::Struct { fields, .. } = type_info {
        for field in fields {
            let item = CompletionItem {
                kind: Some(CompletionItemKind::FIELD),
                label: field.name.as_str().to_string(),
                label_details: Some(CompletionItemLabelDetails {
                    description: Some(field.type_argument.span.str()),
                    detail: None,
                }),
                ..Default::default()
            };
            completion_items.push(item);
        }
    }

    for method in namespace.get_methods_for_type(engines, type_id) {
        let params = parameters_of_fn(engines, &method);

        // Only show methods that take `self` as the first parameter.
        if params
            .first()
            .and_then(|p| Some(p.is_self()))
            .unwrap_or_default()
        {
            let params_short = match params.is_empty() {
                true => "()".to_string(),
                false => "(…)".to_string(),
            };
            let params_str = params
                .iter()
                .filter_map(|p| {
                    if p.is_self() {
                        return None;
                    }
                    Some(p.name.as_str())
                })
                .collect::<Vec<&str>>()
                .join(", ");
            let decl_string = method.decl_span.clone().str();
            let signature = decl_string
                .split_at(decl_string.find("{").unwrap())
                .0
                .split_at(decl_string.find("fn").unwrap())
                .1
                .to_string();

            let item = CompletionItem {
                kind: Some(CompletionItemKind::METHOD),
                label: format!("{}{}", method.name.as_str(), params_short),
                text_edit: Some(CompletionTextEdit::Edit(TextEdit {
                    range: Range {
                        start: position,
                        end: position,
                    },
                    new_text: format!("{}({})", method.name.as_str(), params_str),
                })),
                label_details: Some(CompletionItemLabelDetails {
                    description: Some(signature),
                    detail: None,
                }),
                ..Default::default()
            };
            completion_items.push(item);
        }
    }

    completion_items
}

/// Returns the [TypeId] of an ident that may include field accesses and may be incomplete.
/// For the first part of the ident, it looks for instantiation in the scope of the given
/// [TyFunctionDeclaration]. For example, given `a.b.c`, it will return the type ID of `c`
/// if it can resolve `a` in the given function.
fn type_id_of_raw_ident(
    engines: Engines,
    namespace: &Items,
    ident: &Ident,
    fn_decl: &TyFunctionDeclaration,
) -> Option<TypeId> {
    let full_ident = ident.as_str();

    // If this ident has no field accesses or chained methods, look for it in the local function scope.
    if !full_ident.contains(".") {
        return type_id_of_local_ident(full_ident, fn_decl);
    }

    // Otherwise, start with the first part of the ident and follow the subsequent types.
    let parts = full_ident.split(".").collect::<Vec<&str>>();
    let mut curr_type_id = type_id_of_local_ident(parts[0], fn_decl);
    let mut i = 1;

    while (i < parts.len()) && curr_type_id.is_some() {
        if parts[i].ends_with(")") {
            let method_name = parts[i].split_at(parts[i].find("(").unwrap_or(0)).0;
            curr_type_id = namespace
                .get_methods_for_type(engines, curr_type_id?)
                .iter()
                .find_map(|decl_ref| {
                    if decl_ref.name.as_str() == method_name {
                        return return_type_id_of_fn(engines, decl_ref);
                    }
                    None
                });
        } else if let TypeInfo::Struct { fields, .. } = engines.te().get(curr_type_id.unwrap()) {
            curr_type_id = fields
                .iter()
                .find(|field| field.name.to_string() == parts[i])
                .and_then(|field| Some(field.type_argument.type_id));
        }
        i += 1;
    }
    curr_type_id
}

/// Returns the [TypeId] of a function or trait function given its [DeclRef].
fn return_type_id_of_fn(engines: Engines, decl_ref: &DeclRef) -> Option<TypeId> {
    return engines
        .de()
        .get_function(&decl_ref.id, &decl_ref.decl_span)
        .ok()
        .and_then(|decl| Some(decl.return_type.type_id))
        .or_else(|| {
            engines
                .de()
                .get_trait_fn(&decl_ref.id, &decl_ref.decl_span)
                .ok()
                .and_then(|decl| Some(decl.return_type))
        });
}

/// Returns the parameters of a function or trait function given its [DeclRef].
fn parameters_of_fn(engines: Engines, decl_ref: &DeclRef) -> Vec<TyFunctionParameter> {
    return engines
        .de()
        .get_function(&decl_ref.id, &decl_ref.decl_span)
        .ok()
        .and_then(|decl| Some(decl.parameters))
        .or_else(|| {
            engines
                .de()
                .get_trait_fn(&decl_ref.id, &decl_ref.decl_span)
                .ok()
                .and_then(|decl| Some(decl.parameters))
        })
        .unwrap_or_default();
}

/// Returns the [TypeId] of an ident by looking for its instantiation within the scope of the
/// given [TyFunctionDeclaration].
fn type_id_of_local_ident(ident_name: &str, fn_decl: &TyFunctionDeclaration) -> Option<TypeId> {
    fn_decl
        .parameters
        .iter()
        .find_map(|param| {
            // Check if this ident is a function parameter
            if param.name.as_str() == ident_name {
                return Some(param.type_argument.type_id);
            }
            None
        })
        .or_else(|| {
            // Check if there is a variable declaration for this ident
            fn_decl.body.contents.iter().find_map(|node| {
                if let TyAstNodeContent::Declaration(TyDeclaration::VariableDeclaration(
                    variable_decl,
                )) = node.content.clone()
                {
                    if variable_decl.name.as_str() == ident_name {
                        return Some(variable_decl.return_type.clone());
                    }
                }
                None
            })
        })
}
