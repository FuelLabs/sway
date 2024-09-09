pub(crate) mod struct_impl;
pub(crate) mod struct_new;

use self::{struct_impl::StructImplCodeAction, struct_new::StructNewCodeAction};
use crate::capabilities::code_actions::{CodeAction, CodeActionContext};
use lsp_types::CodeActionOrCommand;
use sway_core::{decl_engine::id::DeclId, language::ty};

use super::common::basic_doc_comment::BasicDocCommentCodeAction;

pub(crate) fn code_actions(
    decl_id: &DeclId<ty::TyStructDecl>,
    ctx: &CodeActionContext,
) -> Vec<CodeActionOrCommand> {
    let decl = (*ctx.engines.de().get_struct(decl_id)).clone();
    vec![
        StructImplCodeAction::new(ctx, &decl).code_action(),
        StructNewCodeAction::new(ctx, &decl).code_action(),
        BasicDocCommentCodeAction::new(ctx, &decl).code_action(),
    ]
}
