pub(crate) mod abi_impl;

use sway_core::{decl_engine::DeclId, Engines};
use sway_types::Spanned;
use tower_lsp::lsp_types::{CodeActionOrCommand, Url};

use self::abi_impl::AbiImplCodeAction;

use super::CodeActionTrait;

pub(crate) fn code_actions(
    engines: Engines<'_>,
    decl_id: &DeclId,
    uri: &Url,
) -> Option<Vec<CodeActionOrCommand>> {
    let decl = engines
        .de()
        .get_abi(decl_id.clone(), &decl_id.span())
        .unwrap();
    // Some(vec![abi_impl::code_action(engines, &decl, uri)])
    Some(vec![
        AbiImplCodeAction::new(engines, &decl, uri).code_action()
    ])
}
