use crate::capabilities::code_actions::{CodeAction, CodeActionContext};
use sway_core::language::ty;
use tower_lsp::lsp_types::CodeActionOrCommand;

use super::common::generate_doc::BasicDocCommentCodeAction;

pub(crate) fn code_actions(
    decl: &ty::TyEnumVariant,
    ctx: CodeActionContext,
) -> Option<Vec<CodeActionOrCommand>> {
    Some(vec![BasicDocCommentCodeAction::new(ctx, decl).code_action()])
}
