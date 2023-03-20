use sway_error::handler::{ErrorEmitted, Handler};

use crate::{language::ty, monomorphize::priv_prelude::*};

pub(crate) fn instruct_root(
    mut ctx: InstructContext,
    handler: &Handler,
    module: &mut ty::TyModule,
) -> Result<(), ErrorEmitted> {
    for (_, submod) in module.submodules.iter_mut() {
        instruct_module(ctx.by_ref(), handler, &mut submod.module)?;
    }
    for node in module.all_nodes.iter() {
        instruct_node(ctx.by_ref(), handler, &node.content)?;
    }
    Ok(())
}

pub(crate) fn instruct_module(
    mut ctx: InstructContext,
    handler: &Handler,
    module: &mut ty::TyModule,
) -> Result<(), ErrorEmitted> {
    for (_, submod) in module.submodules.iter_mut() {
        instruct_module(ctx.by_ref(), handler, &mut submod.module)?;
    }
    for node in module.all_nodes.iter() {
        instruct_node(ctx.by_ref(), handler, &node.content)?;
    }
    Ok(())
}
