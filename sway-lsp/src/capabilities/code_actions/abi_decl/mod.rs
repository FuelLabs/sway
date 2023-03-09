pub(crate) mod abi_impl;

use sway_core::decl_engine::DeclRefAbi;
use tower_lsp::lsp_types::CodeActionOrCommand;

use self::abi_impl::AbiImplCodeAction;

use super::{CodeAction, CodeActionContext};

pub(crate) fn code_actions(
    decl_ref: &DeclRefAbi,
    ctx: CodeActionContext,
) -> Option<Vec<CodeActionOrCommand>> {
    let decl = ctx.engines.de().get_abi(decl_ref);
    Some(vec![AbiImplCodeAction::new(ctx, &decl).code_action()])
}
