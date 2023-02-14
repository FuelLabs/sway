pub(crate) mod abi_impl;

use sway_core::decl_engine::id::DeclId;
use sway_types::Span;
use tower_lsp::lsp_types::CodeActionOrCommand;

use self::abi_impl::AbiImplCodeAction;

use super::{CodeAction, CodeActionContext};

pub(crate) fn code_actions(
    decl_id: &DeclId,
    decl_span: &Span,
    ctx: CodeActionContext,
) -> Option<Vec<CodeActionOrCommand>> {
    let decl = ctx.engines.de().get_abi(decl_id, decl_span).ok()?;
    Some(vec![AbiImplCodeAction::new(ctx, &decl).code_action()])
}
