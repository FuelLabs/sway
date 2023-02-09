use std::collections::HashMap;

use sway_core::{
    language::ty::{TyAbiDeclaration, TyFunctionParameter, TyTraitFn},
    Engines,
};
use sway_types::Spanned;
use tower_lsp::lsp_types::{CodeActionOrCommand, TextEdit, Url};

use crate::capabilities::code_actions::{CodeActionTrait, CODE_ACTION_IMPL_TITLE, CONTRACT, TAB};

pub(crate) struct AbiImplCodeAction<'a> {
    engines: Engines<'a>,
    decl: &'a TyAbiDeclaration,
    uri: &'a Url,
}

impl<'a> CodeActionTrait<'a, TyAbiDeclaration> for AbiImplCodeAction<'a> {
    fn new(engines: Engines<'a>, decl: &'a TyAbiDeclaration, uri: &'a Url) -> Self {
        Self { engines, decl, uri }
    }

    fn code_action(&self) -> CodeActionOrCommand {
        let text_edit = TextEdit {
            range: Self::range_after_last_line(&self.decl.span),
            new_text: self.new_text(),
        };

        self.build_code_action(
            self.title(),
            HashMap::from([(self.uri.clone(), vec![text_edit])]),
            self.uri,
        )
    }

    fn new_text(&self) -> String {
        self.impl_string(
            None,
            self.fn_signatures_string(),
            Some(CONTRACT.to_string()),
        )
    }

    fn title(&self) -> String {
        format!("{} `{}`", CODE_ACTION_IMPL_TITLE, self.decl_name())
    }

    fn decl_name(&self) -> String {
        self.decl.name.to_string()
    }
}

impl AbiImplCodeAction<'_> {
    fn param_string(&self, param: &TyFunctionParameter) -> String {
        format!("{}: {}", param.name, param.type_span.as_str())
    }

    fn return_type_string(&self, function_decl: TyTraitFn) -> String {
        let type_engine = self.engines.te();
        // Unit is the implicit return type for ABI functions.
        if type_engine.get(function_decl.return_type).is_unit() {
            String::from("")
        } else {
            format!(" -> {}", function_decl.return_type_span.as_str())
        }
    }

    fn fn_signature_string(&self, function_decl: TyTraitFn) -> String {
        let param_string: String = function_decl
            .parameters
            .iter()
            .map(|param| self.param_string(param))
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
            self.return_type_string(function_decl)
        )
    }

    fn fn_signatures_string(&self) -> String {
        let decl_engine = self.engines.de();
        self.decl
            .interface_surface
            .iter()
            .filter_map(|function_decl_id| {
                decl_engine
                    .get_trait_fn(function_decl_id.clone(), &function_decl_id.span())
                    .ok()
                    .map(|function_decl| self.fn_signature_string(function_decl))
            })
            .collect::<Vec<String>>()
            .join("\n")
    }
}
