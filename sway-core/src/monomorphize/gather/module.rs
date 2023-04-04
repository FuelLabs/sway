use crate::{language::ty, monomorphize::priv_prelude::*};

pub(crate) fn gather_from_root(ctx: GatherContext, module: &ty::TyModule) {
    for (_, submod) in module.submodules_recursive() {
        gather_from_module(ctx, &submod.module);
    }
    for node in module.all_nodes.iter() {
        gather_from_node(ctx, &node.content);
    }
}

pub(crate) fn gather_from_module(ctx: GatherContext, module: &ty::TyModule) {
    for (_, submod) in module.submodules_recursive() {
        gather_from_module(ctx, &submod.module);
    }
    for node in module.all_nodes.iter() {
        gather_from_node(ctx, &node.content);
    }
}
