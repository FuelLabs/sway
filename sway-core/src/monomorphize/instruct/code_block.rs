use sway_error::handler::{ErrorEmitted, Handler};

use crate::{language::ty, monomorphize::priv_prelude::*};

pub(crate) fn instruct_code_block(
    mut ctx: InstructContext,
    handler: &Handler,
    body: &ty::TyCodeBlock,
) -> Result<(), ErrorEmitted> {
    body.contents
        .iter()
        .try_for_each(|node| instruct_node(ctx.by_ref(), handler, &node.content))?;

    Ok(())
}
