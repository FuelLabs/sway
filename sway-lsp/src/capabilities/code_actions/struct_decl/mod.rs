pub(crate) mod struct_impl;
pub(crate) mod struct_new;

use sway_core::{decl_engine::DeclId, Engines};
use sway_types::Spanned;
use tower_lsp::lsp_types::{CodeActionOrCommand, Url};

use self::{struct_impl::StructImplCodeAction, struct_new::StructNewCodeAction};

use super::CodeActionTrait;

pub(crate) fn code_actions(
    engines: Engines<'_>,
    decl_id: &DeclId,
    uri: &Url,
) -> Option<Vec<CodeActionOrCommand>> {
    let decl = engines
        .de()
        .get_struct(decl_id.clone(), &decl_id.span())
        .unwrap();
    Some(vec![
        StructImplCodeAction::new(engines, &decl, uri).code_action(),
        StructNewCodeAction::new(engines, &decl, uri).code_action(),
    ])
}
