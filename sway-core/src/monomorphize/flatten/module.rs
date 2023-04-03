use crate::{language::ty::*, monomorphize::priv_prelude::*, Engines};

pub(crate) fn flatten_root(engines: Engines<'_>, module: TyModule) -> TyModule {
    for (_, submod) in module.submodules.iter_mut() {
        flatten_module(engines, &mut submod.module);
    }
    for node in module.all_nodes.iter_mut() {
        flatten_node(engines, &mut node.content);
    }
}

pub(crate) fn flatten_module(engines: Engines<'_>, module: TyModule) -> TyModule {
    for (_, submod) in module.submodules.iter_mut() {
        flatten_module(engines, &mut submod.module);
    }
    for node in module.all_nodes.iter_mut() {
        flatten_node(engines, &mut node.content);
    }
}
