pub(crate) mod struct_impl;
pub(crate) mod struct_new;

use sway_core::decl_engine::DeclId;
use sway_types::Spanned;
use tower_lsp::lsp_types::CodeActionOrCommand;

use self::{struct_impl::StructImplCodeAction, struct_new::StructNewCodeAction};

use super::{CodeAction, CodeActionContext};

pub(crate) fn code_actions(
    decl_id: &DeclId,
    ctx: CodeActionContext,
) -> Option<Vec<CodeActionOrCommand>> {
    let decl = ctx
        .engines
        .de()
        .get_struct(decl_id.clone(), &decl_id.span())
        .unwrap();
    Some(vec![
        StructImplCodeAction::new(ctx.clone(), &decl).code_action(),
        StructNewCodeAction::new(ctx, &decl).code_action(),
    ])
}
