use sway_error::handler::{ErrorEmitted, Handler};

use crate::{language::ty, monomorphize::priv_prelude::*};

pub(crate) fn gather_from_root(
    mut ctx: GatherContext,
    handler: &Handler,
    module: &ty::TyModule,
) -> Result<(), ErrorEmitted> {
    for (_, submod) in module.submodules_recursive() {
        gather_from_module(ctx.by_ref(), handler, &submod.module)?;
    }
    for node in module.all_nodes.iter() {
        gather_from_node(ctx.by_ref(), handler, &node.content)?;
    }
    Ok(())
}

pub(crate) fn gather_from_module(
    ctx: GatherContext,
    handler: &Handler,
    module: &ty::TyModule,
) -> Result<(), ErrorEmitted> {
    let module_namespace = ctx.namespace.new_with_module(&module.namespace);
    let mut ctx = ctx.scoped(&module_namespace);
    for (_, submod) in module.submodules_recursive() {
        gather_from_module(ctx.by_ref(), handler, &submod.module)?;
    }
    for node in module.all_nodes.iter() {
        gather_from_node(ctx.by_ref(), handler, &node.content)?;
    }
    Ok(())
}
