use crate::capabilities::code_actions::{CodeAction, CodeActionContext, CODE_ACTION_DOC_TITLE};
use lsp_types::{Range, Url};
use sway_types::Spanned;

use super::generate_doc::GenerateDocCodeAction;

pub struct BasicDocCommentCodeAction<'a, T: Spanned> {
    decl: &'a T,
    uri: &'a Url,
}

impl<'a, T: Spanned> GenerateDocCodeAction<'a, T> for BasicDocCommentCodeAction<'a, T> {}

impl<'a, T: Spanned> CodeAction<'a, T> for BasicDocCommentCodeAction<'a, T> {
    fn new(ctx: CodeActionContext<'a>, decl: &'a T) -> Self {
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

    fn decl(&self) -> &T {
        self.decl
    }

    fn uri(&self) -> &Url {
        self.uri
    }
}
