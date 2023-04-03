use crate::{language::ty, monomorphize::priv_prelude::*};

pub(crate) fn gather_from_root(mut ctx: GatherContext, module: &ty::TyModule) {
    for (_, submod) in module.submodules_recursive() {
        gather_from_module(ctx.by_ref(), &submod.module);
    }
    for node in module.all_nodes.iter() {
        gather_from_node(ctx.by_ref(), &node.content);
    }
}

pub(crate) fn gather_from_module(ctx: GatherContext, module: &ty::TyModule) {
    let module_namespace = ctx.namespace.new_with_module(&module.namespace);
    let mut ctx = ctx.scoped(&module_namespace);
    for (_, submod) in module.submodules_recursive() {
        gather_from_module(ctx.by_ref(), &submod.module);
    }
    for node in module.all_nodes.iter() {
        gather_from_node(ctx.by_ref(), &node.content);
    }
}
