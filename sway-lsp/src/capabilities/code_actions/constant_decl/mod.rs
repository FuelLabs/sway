use crate::capabilities::code_actions::{CodeAction, CodeActionContext};
use lsp_types::CodeActionOrCommand;
use sway_core::language::ty;

use super::common::basic_doc_comment::BasicDocCommentCodeAction;

pub(crate) fn code_actions(
    decl: &ty::TyConstantDecl,
    ctx: CodeActionContext,
) -> Option<Vec<CodeActionOrCommand>> {
    Some(vec![BasicDocCommentCodeAction::new(ctx, decl).code_action()])
}
