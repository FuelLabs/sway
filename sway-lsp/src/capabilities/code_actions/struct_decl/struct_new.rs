use sway_core::{
    language::ty::{TyStructDeclaration, TyStructField},
    Engines,
};
use tower_lsp::lsp_types::Url;

use crate::capabilities::code_actions::{CodeActionTrait, CODE_ACTION_NEW_TITLE, TAB};

pub(crate) struct StructNewCodeAction<'a> {
    decl: &'a TyStructDeclaration,
    uri: &'a Url,
}

impl<'a> CodeActionTrait<'a, TyStructDeclaration> for StructNewCodeAction<'a> {
    fn new(_engines: Engines<'a>, decl: &'a TyStructDeclaration, uri: &'a Url) -> Self {
        Self { decl, uri }
    }

    fn new_text(&self) -> String {
        let params = self.params_string(&self.decl.fields);
        let new_fn = self.fn_signature_string(
            "new".to_string(),
            params,
            &self.decl.attributes,
            self.return_type_string(),
            Some(self.fn_body()),
        );
        self.impl_string(
            self.type_param_string(&self.decl.type_parameters),
            format!("{TAB}{new_fn}\n"),
            None,
        )
    }

    fn title(&self) -> String {
        CODE_ACTION_NEW_TITLE.to_string()
    }

    fn decl_name(&self) -> String {
        self.decl.call_path.suffix.to_string()
    }

    fn decl(&self) -> &TyStructDeclaration {
        self.decl
    }

    fn uri(&self) -> &Url {
        self.uri
    }
}

impl StructNewCodeAction<'_> {
    fn return_type_string(&self) -> String {
        " -> Self".to_string()
    }

    fn params_string(&self, params: &[TyStructField]) -> String {
        params
            .iter()
            .map(|field| format!("{}: {}", field.name, field.type_span.as_str()))
            .collect::<Vec<String>>()
            .join(", ")
    }

    fn fn_body(&self) -> String {
        format!(
            "Self {{ {} }}",
            self.decl
                .fields
                .iter()
                .map(|field| format!("{}", field.name))
                .collect::<Vec<String>>()
                .join(", ")
        )
    }
}
