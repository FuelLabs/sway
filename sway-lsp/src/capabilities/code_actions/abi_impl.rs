use std::collections::HashMap;

use serde_json::Value;
use sway_core::{
    declaration_engine,
    language::ty::{TyAbiDeclaration, TyFunctionParameter, TyTraitFn},
};
use sway_types::Spanned;
use tower_lsp::lsp_types::{
    CodeAction, CodeActionKind, CodeActionOrCommand, Position, Range, TextEdit, Url, WorkspaceEdit,
};

const CODE_ACTION_DESCRIPTION: &str = "Generate impl for contract";
const TAB: &str = "    ";

pub(crate) fn abi_impl_code_action(abi_decl: TyAbiDeclaration, uri: Url) -> CodeActionOrCommand {
    let (last_line, _) = abi_decl.span.end_pos().line_col();
    let insertion_position = Position {
        line: last_line as u32,
        character: 0,
    };
    let text_edit = TextEdit {
        range: Range {
            start: insertion_position,
            end: insertion_position,
        },
        new_text: get_contract_impl_string(abi_decl),
    };
    let mut text_edit_map = HashMap::new();
    text_edit_map.insert(uri.clone(), vec![text_edit]);

    CodeActionOrCommand::CodeAction(CodeAction {
        title: String::from(CODE_ACTION_DESCRIPTION),
        kind: Some(CodeActionKind::REFACTOR),
        edit: Some(WorkspaceEdit {
            changes: Some(text_edit_map),
            ..Default::default()
        }),
        data: Some(Value::String(uri.to_string())),
        ..Default::default()
    })
}

fn get_param_string(param: &TyFunctionParameter) -> String {
    format!("{}: {}", param.name, param.type_span.as_str())
}

fn get_return_type_string(function_decl: TyTraitFn) -> String {
    // When the method is missing a return type, the compiler sets the return type span
    // to the full function signature. This is a hacky way of checking whether the
    // actual return type is provided.
    let return_type_span = function_decl.return_type_span.as_str();
    if return_type_span.contains("fn ") {
        String::from("")
    } else {
        format!(" -> {}", return_type_span)
    }
}

fn get_function_signatures(abi_decl: TyAbiDeclaration) -> String {
    abi_decl
        .interface_surface
        .iter()
        .filter_map(|function_decl_id| {
            declaration_engine::de_get_trait_fn(function_decl_id.clone(), &function_decl_id.span())
                .ok()
                .map(|function_decl| {
                    let param_string: String = function_decl
                        .parameters
                        .iter()
                        .map(get_param_string)
                        .collect::<Vec<String>>()
                        .join(", ");
                    let attribute_string = function_decl
                        .attributes
                        .iter()
                        .map(|(_, attrs)| {
                            attrs
                                .iter()
                                .map(|attr| format!("{}{}", TAB, attr.span.as_str()))
                                .collect::<Vec<String>>()
                                .join("\n")
                        })
                        .collect::<Vec<String>>()
                        .join("\n");
                    let attribute_prefix = match attribute_string.len() > 1 {
                        true => "\n",
                        false => "",
                    };
                    format!(
                        "{}{}\n{}fn {}({}){} {{}}",
                        attribute_prefix,
                        attribute_string,
                        TAB,
                        function_decl.name.clone(),
                        param_string,
                        get_return_type_string(function_decl)
                    )
                })
        })
        .collect::<Vec<String>>()
        .join("\n")
}

fn get_contract_impl_string(abi_decl: TyAbiDeclaration) -> String {
    let contract_name = abi_decl.name.to_string();
    format!(
        "\nimpl {} for Contract {{{}\n}}\n",
        contract_name,
        get_function_signatures(abi_decl).as_str()
    )
}
