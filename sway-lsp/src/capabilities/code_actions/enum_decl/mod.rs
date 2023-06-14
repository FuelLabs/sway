pub(crate) mod doc_comment;
pub(crate) mod enum_impl;

use self::enum_impl::EnumImplCodeAction;
use crate::capabilities::code_actions::{CodeAction, CodeActionContext};
use lsp_types::CodeActionOrCommand;
use sway_core::{decl_engine::id::DeclId, language::ty};

use super::common::generate_doc::BasicDocCommentCodeAction;

pub(crate) fn code_actions(
    decl_id: &DeclId<ty::TyEnumDecl>,
    ctx: CodeActionContext,
) -> Option<Vec<CodeActionOrCommand>> {
    let decl = ctx.engines.de().get_enum(decl_id);
    Some(vec![
        EnumImplCodeAction::new(ctx.clone(), &decl).code_action(),
        BasicDocCommentCodeAction::new(ctx, &decl).code_action(),
    ])
}
