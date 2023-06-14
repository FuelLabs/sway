use crate::capabilities::code_actions::{
    common::generate_doc::GenerateDocCodeAction, CodeAction, CodeActionContext,
    CODE_ACTION_DOC_TITLE,
};
use lsp_types::{Range, Url};
use sway_core::language::ty::TyEnumDecl;

pub(crate) struct DocCommentCodeAction<'a> {
    decl: &'a TyEnumDecl,
    uri: &'a Url,
}

impl<'a> GenerateDocCodeAction<'a, TyEnumDecl> for DocCommentCodeAction<'a> {}

impl<'a> CodeAction<'a, TyEnumDecl> for DocCommentCodeAction<'a> {
    fn new(ctx: CodeActionContext<'a>, decl: &'a TyEnumDecl) -> Self {
        Self { decl, uri: ctx.uri }
    }

    fn new_text(&self) -> String {
        self.default_template()
    }

    fn range(&self) -> Range {
        self.range_before()
    }

    fn title(&self) -> String {
        CODE_ACTION_DOC_TITLE.to_string()
    }

    fn decl(&self) -> &TyEnumDecl {
        self.decl
    }

    fn uri(&self) -> &Url {
        self.uri
    }
}
