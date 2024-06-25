use crate::capabilities::code_actions::{CodeAction, CodeActionContext};
use sway_core::language::ty;
use tower_lsp::lsp_types::CodeActionOrCommand;

use super::common::fn_doc_comment::FnDocCommentCodeAction;

pub(crate) fn code_actions(
    decl: &ty::TyTraitFn,
    ctx: &CodeActionContext,
) -> Vec<CodeActionOrCommand> {
    vec![FnDocCommentCodeAction::new(ctx, decl).code_action()]
}
