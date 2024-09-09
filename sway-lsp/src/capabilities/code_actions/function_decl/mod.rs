use crate::capabilities::code_actions::{CodeAction, CodeActionContext};
use lsp_types::CodeActionOrCommand;
use sway_core::language::ty;

use super::common::fn_doc_comment::FnDocCommentCodeAction;

pub(crate) fn code_actions(
    decl: &ty::TyFunctionDecl,
    ctx: &CodeActionContext,
) -> Vec<CodeActionOrCommand> {
    vec![FnDocCommentCodeAction::new(ctx, decl).code_action()]
}
