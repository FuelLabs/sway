use std::collections::HashMap;

use sway_core::{
    language::ty::{TyAbiDeclaration, TyFunctionParameter, TyTraitFn},
    Engines,
};
use sway_types::Spanned;
use tower_lsp::lsp_types::{CodeActionOrCommand, TextEdit, Url};

use crate::capabilities::code_actions::{build_code_action, range_after_last_line, TAB};

const CODE_ACTION_DESCRIPTION: &str = "Generate impl for contract";

pub(crate) fn code_action(
    engines: Engines<'_>,
    decl: &TyAbiDeclaration,
    uri: &Url,
) -> CodeActionOrCommand {
    let text_edit = TextEdit {
        range: range_after_last_line(&decl.span),
        new_text: get_contract_impl_string(engines, decl),
    };

    build_code_action(
        CODE_ACTION_DESCRIPTION.to_string(),
        HashMap::from([(uri.clone(), vec![text_edit])]),
        &uri,
    )
}

fn get_param_string(param: &TyFunctionParameter) -> String {
    format!("{}: {}", param.name, param.type_span.as_str())
}

fn get_return_type_string(engines: Engines<'_>, function_decl: TyTraitFn) -> String {
    let type_engine = engines.te();
    // Unit is the implicit return type for ABI functions.
    if type_engine.get(function_decl.return_type).is_unit() {
        String::from("")
    } else {
        format!(" -> {}", function_decl.return_type_span.as_str())
    }
}

fn get_function_signatures(engines: Engines<'_>, abi_decl: &TyAbiDeclaration) -> String {
    let decl_engine = engines.de();
    abi_decl
        .interface_surface
        .iter()
        .filter_map(|function_decl_id| {
            decl_engine
                .get_trait_fn(function_decl_id.clone(), &function_decl_id.span())
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
                        get_return_type_string(engines, function_decl)
                    )
                })
        })
        .collect::<Vec<String>>()
        .join("\n")
}

fn get_contract_impl_string(engines: Engines<'_>, abi_decl: &TyAbiDeclaration) -> String {
    let contract_name = abi_decl.name.to_string();
    format!(
        "\nimpl {} for Contract {{{}\n}}\n",
        contract_name,
        get_function_signatures(engines, abi_decl).as_str()
    )
}
