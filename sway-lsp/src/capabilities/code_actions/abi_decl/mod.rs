pub(crate) mod abi_impl;

use sway_core::{decl_engine::DeclId, Engines};
use sway_types::Spanned;
use tower_lsp::lsp_types::{CodeActionOrCommand, Url};

pub(crate) fn code_actions(
    engines: Engines<'_>,
    decl_id: &DeclId,
    uri: &Url,
) -> Option<Vec<CodeActionOrCommand>> {
    let decl = engines
        .de()
        .get_abi(decl_id.clone(), &decl_id.span())
        .unwrap();
    Some(vec![abi_impl::code_action(engines, &decl, uri)])
}
