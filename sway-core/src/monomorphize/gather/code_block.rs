use sway_error::handler::{ErrorEmitted, Handler};

use crate::{language::ty, monomorphize::priv_prelude::*};

pub(crate) fn gather_from_code_block(
    mut ctx: GatherContext,
    handler: &Handler,
    body: &ty::TyCodeBlock,
) -> Result<(), ErrorEmitted> {
    body.contents
        .iter()
        .try_for_each(|node| gather_from_node(ctx.by_ref(), handler, &node.content))?;

    Ok(())
}
