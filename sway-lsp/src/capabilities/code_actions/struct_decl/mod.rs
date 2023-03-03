pub(crate) mod struct_impl;
pub(crate) mod struct_new;

use self::{struct_impl::StructImplCodeAction, struct_new::StructNewCodeAction};
use crate::capabilities::code_actions::{CodeAction, CodeActionContext};
use sway_core::{decl_engine::id::DeclId, language::ty::TyStructDeclaration};
use tower_lsp::lsp_types::CodeActionOrCommand;

pub(crate) fn code_actions(
    decl_id: &DeclId<TyStructDeclaration>,
    ctx: CodeActionContext,
) -> Option<Vec<CodeActionOrCommand>> {
    let decl = ctx.engines.de().get_struct(decl_id);
    Some(vec![
        StructImplCodeAction::new(ctx.clone(), &decl).code_action(),
        StructNewCodeAction::new(ctx, &decl).code_action(),
    ])
}
