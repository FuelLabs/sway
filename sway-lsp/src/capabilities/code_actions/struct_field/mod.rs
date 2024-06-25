use crate::capabilities::code_actions::{CodeAction, CodeActionContext};
use sway_core::language::ty;
use tower_lsp::lsp_types::CodeActionOrCommand;

use super::common::basic_doc_comment::BasicDocCommentCodeAction;

pub(crate) fn code_actions(
    decl: &ty::TyStructField,
    ctx: &CodeActionContext,
) -> Vec<CodeActionOrCommand> {
    vec![BasicDocCommentCodeAction::new(ctx, decl).code_action()]
}
