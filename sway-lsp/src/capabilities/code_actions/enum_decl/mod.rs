pub(crate) mod enum_impl;

use self::enum_impl::EnumImplCodeAction;
use crate::capabilities::code_actions::{CodeAction, CodeActionContext};
use sway_core::{decl_engine::id::DeclId, language::ty};
use tower_lsp::lsp_types::CodeActionOrCommand;

use super::common::basic_doc_comment::BasicDocCommentCodeAction;

pub(crate) fn code_actions(
    decl_id: &DeclId<ty::TyEnumDecl>,
    ctx: &CodeActionContext,
) -> Vec<CodeActionOrCommand> {
    let decl = (*ctx.engines.de().get_enum(decl_id)).clone();
    vec![
        EnumImplCodeAction::new(ctx, &decl).code_action(),
        BasicDocCommentCodeAction::new(ctx, &decl).code_action(),
    ]
}
