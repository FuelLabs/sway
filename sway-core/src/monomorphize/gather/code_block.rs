use crate::{language::ty, monomorphize::priv_prelude::*};

pub(crate) fn gather_from_code_block(mut ctx: GatherContext, body: &ty::TyCodeBlock) {
    body.contents
        .iter()
        .for_each(|node| gather_from_node(ctx.by_ref(), &node.content));
}
