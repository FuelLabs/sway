use crate::capabilities::code_actions::{
    common::generate_impl::{GenerateImplCodeAction, TAB},
    CodeAction, CodeActionContext, CODE_ACTION_IMPL_TITLE,
};
use lsp_types::{Range, Url};
use sway_core::language::ty::TyStructDecl;

pub(crate) struct StructImplCodeAction<'a> {
    decl: &'a TyStructDecl,
    uri: &'a Url,
}

impl<'a> GenerateImplCodeAction<'a, TyStructDecl> for StructImplCodeAction<'a> {
    fn decl_name(&self) -> String {
        self.decl.call_path.suffix.to_string()
    }
}

impl<'a> CodeAction<'a, TyStructDecl> for StructImplCodeAction<'a> {
    fn new(ctx: &CodeActionContext<'a>, decl: &'a TyStructDecl) -> Self {
        Self { decl, uri: ctx.uri }
    }

    fn new_text(&self) -> String {
        self.impl_string(
            self.type_param_string(&self.decl.generic_parameters),
            format!("\n{TAB}\n"),
            None,
        )
    }

    fn title(&self) -> String {
        format!("{} `{}`", CODE_ACTION_IMPL_TITLE, self.decl_name())
    }

    fn range(&self) -> Range {
        self.range_after()
    }

    fn decl(&self) -> &TyStructDecl {
        self.decl
    }

    fn uri(&self) -> &Url {
        self.uri
    }
}
