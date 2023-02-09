use std::collections::HashMap;

use serde_json::Value;
use sway_core::{language::ty::TyStructDeclaration, Engines};
use sway_types::Span;
use tower_lsp::lsp_types::{
    CodeAction, CodeActionKind, CodeActionOrCommand, Position, Range, TextEdit, Url, WorkspaceEdit,
};

use crate::capabilities::code_actions::{CodeActionTrait, CODE_ACTION_IMPL_TITLE, TAB};

pub(crate) struct StructImplCodeAction<'a> {
    decl: &'a TyStructDeclaration,
    uri: &'a Url,
}

impl<'a> CodeActionTrait<'a, TyStructDeclaration> for StructImplCodeAction<'a> {
    fn new(_engines: Engines<'a>, decl: &'a TyStructDeclaration, uri: &'a Url) -> Self {
        Self { decl, uri }
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
        self.impl_string(self.type_param_string(), format!("\n{}\n", TAB), None)
    }

    fn title(&self) -> String {
        format!("{} `{}`", CODE_ACTION_IMPL_TITLE, self.decl_name())
    }

    fn decl_name(&self) -> String {
        self.decl.call_path.suffix.to_string()
    }
}

impl StructImplCodeAction<'_> {
    fn type_param_string(&self) -> Option<String> {
        if self.decl.type_parameters.is_empty() {
            None
        } else {
            Some(
                self.decl
                    .type_parameters
                    .iter()
                    .map(|param| param.name_ident.to_string())
                    .collect::<Vec<_>>()
                    .join(", "),
            )
        }
    }
}

// const CODE_ACTION_DESCRIPTION: &str = "Generate impl for struct";

// pub(crate) fn code_action(decl: &TyStructDeclaration, uri: &Url) -> CodeActionOrCommand {
//     let text_edit = TextEdit {
//         range: range_after_last_line(&decl.span),
//         new_text: get_impl_string(decl),
//     };

//     build_code_action(
//         CODE_ACTION_DESCRIPTION.to_string(),
//         HashMap::from([(uri.clone(), vec![text_edit])]),
//         &uri,
//     )
// }

// fn get_impl_string(struct_decl: &TyStructDeclaration) -> String {
//     let decl_name = struct_decl.call_path.suffix.as_str();
//     format!("\nimpl {} {{\n{}\n}}\n", decl_name, TAB)
// }
