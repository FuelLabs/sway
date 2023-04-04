use crate::{language::ty::*, monomorphize::priv_prelude::*, Engines};

pub(crate) fn find_from_root<'a>(engines: Engines<'_>, module: &'a TyModule) -> Findings<'a> {
    module
        .submodules_recursive()
        .map(|(_, submod)| find_from_module(engines, &submod.module))
        .chain(
            module
                .all_nodes
                .iter()
                .map(|node| find_from_node(engines, &node.content)),
        )
        .collect()
}

pub(crate) fn find_from_module<'a>(engines: Engines<'_>, module: &'a TyModule) -> Findings<'a> {
    module
        .submodules_recursive()
        .map(|(_, submod)| find_from_module(engines, &submod.module))
        .chain(
            module
                .all_nodes
                .iter()
                .map(|node| find_from_node(engines, &node.content)),
        )
        .collect()
}
