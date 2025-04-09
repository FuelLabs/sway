use crate::capabilities::code_actions::{
    common::generate_impl::{GenerateImplCodeAction, TAB},
    CodeAction, CodeActionContext, CODE_ACTION_IMPL_TITLE,
};
use lsp_types::{Range, Url};
use sway_core::language::ty::TyEnumDecl;

pub(crate) struct EnumImplCodeAction<'a> {
    decl: &'a TyEnumDecl,
    uri: &'a Url,
}

impl<'a> GenerateImplCodeAction<'a, TyEnumDecl> for EnumImplCodeAction<'a> {
    fn decl_name(&self) -> String {
        self.decl.call_path.suffix.to_string()
    }
}

impl<'a> CodeAction<'a, TyEnumDecl> for EnumImplCodeAction<'a> {
    fn new(ctx: &CodeActionContext<'a>, decl: &'a TyEnumDecl) -> Self {
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

    fn decl(&self) -> &TyEnumDecl {
        self.decl
    }

    fn uri(&self) -> &Url {
        self.uri
    }
}
