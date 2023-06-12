pub(crate) mod doc_comment;

use self::doc_comment::DocCommentCodeAction;
use crate::capabilities::code_actions::{CodeAction, CodeActionContext};
use sway_core::language::ty;
use tower_lsp::lsp_types::CodeActionOrCommand;

pub(crate) fn code_actions(
    decl: &ty::TyFunctionDecl,
    ctx: CodeActionContext,
) -> Option<Vec<CodeActionOrCommand>> {
    Some(vec![DocCommentCodeAction::new(ctx, decl).code_action()])
}
