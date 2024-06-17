pub(crate) mod abi_impl;

use self::abi_impl::AbiImplCodeAction;
use super::{CodeAction, CodeActionContext};
use lsp_types::CodeActionOrCommand;
use sway_core::{decl_engine::id::DeclId, language::ty::TyAbiDecl};

pub(crate) fn code_actions(
    decl_id: &DeclId<TyAbiDecl>,
    ctx: &CodeActionContext,
) -> Vec<CodeActionOrCommand> {
    let decl = ctx.engines.de().get_abi(decl_id);
    vec![AbiImplCodeAction::new(ctx, &decl).code_action()]
}
