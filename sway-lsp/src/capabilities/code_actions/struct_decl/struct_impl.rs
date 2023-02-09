use sway_core::{language::ty::TyStructDeclaration, Engines};
use tower_lsp::lsp_types::Url;

use crate::capabilities::code_actions::{CodeActionTrait, CODE_ACTION_IMPL_TITLE, TAB};

pub(crate) struct StructImplCodeAction<'a> {
    decl: &'a TyStructDeclaration,
    uri: &'a Url,
}

impl<'a> CodeActionTrait<'a, TyStructDeclaration> for StructImplCodeAction<'a> {
    fn new(_engines: Engines<'a>, decl: &'a TyStructDeclaration, uri: &'a Url) -> Self {
        Self { decl, uri }
    }

    fn new_text(&self) -> String {
        self.impl_string(
            self.type_param_string(&self.decl.type_parameters),
            format!("\n{TAB}\n"),
            None,
        )
    }

    fn title(&self) -> String {
        format!("{} `{}`", CODE_ACTION_IMPL_TITLE, self.decl_name())
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
