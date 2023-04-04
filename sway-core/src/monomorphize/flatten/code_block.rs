use crate::{language::ty, monomorphize::priv_prelude::*, Engines};

use super::findings::Findings;

pub(crate) fn find_from_code_block<'a>(
    engines: Engines<'_>,
    body: &'a ty::TyCodeBlock,
) -> Findings<'a> {
    body.contents
        .iter()
        .map(|node| find_from_node(engines, &node.content))
        .collect()
}
