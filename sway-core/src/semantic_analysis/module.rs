use crate::{
    error::*,
    language::{parsed::*, ty, DepName},
    semantic_analysis::*,
};

impl ty::TyModule {
    /// Type-check the given parsed module to produce a typed module.
    ///
    /// Recursively type-checks submodules first.
    pub fn type_check(mut ctx: TypeCheckContext, parsed: &ParseModule) -> CompileResult<Self> {
        let ParseModule { submodules, tree } = parsed;

        // Type-check submodules first in order of declaration.
        let mut submodules_res = ok(vec![], vec![], vec![]);
        for (name, submodule) in submodules {
            let submodule_res = ty::TySubmodule::type_check(ctx.by_ref(), name.clone(), submodule);
            submodules_res = submodules_res.flat_map(|mut submodules| {
                submodule_res.map(|submodule| {
                    submodules.push((name.clone(), submodule));
                    submodules
                })
            });
        }

        // TODO: Ordering should be solved across all modules prior to the beginning of type-check.
        let ordered_nodes_res = node_dependencies::order_ast_nodes_by_dependency(
            ctx.type_engine,
            tree.root_nodes.clone(),
        );

        let typed_nodes_res = ordered_nodes_res
            .flat_map(|ordered_nodes| Self::type_check_nodes(ctx.by_ref(), ordered_nodes));

        submodules_res.flat_map(|submodules| {
            typed_nodes_res.map(|all_nodes| Self {
                submodules,
                namespace: ctx.namespace.module().clone(),
                all_nodes,
            })
        })
    }

    fn type_check_nodes(
        mut ctx: TypeCheckContext,
        nodes: Vec<AstNode>,
    ) -> CompileResult<Vec<ty::TyAstNode>> {
        let mut warnings = Vec::new();
        let mut errors = Vec::new();
        let typed_nodes = nodes
            .into_iter()
            .map(|node| ty::TyAstNode::type_check(ctx.by_ref(), node))
            .filter_map(|res| res.ok(&mut warnings, &mut errors))
            .collect();
        ok(typed_nodes, warnings, errors)
    }
}

impl ty::TySubmodule {
    pub fn type_check(
        parent_ctx: TypeCheckContext,
        dep_name: DepName,
        submodule: &ParseSubmodule,
    ) -> CompileResult<Self> {
        let ParseSubmodule {
            library_name,
            module,
        } = submodule;
        parent_ctx.enter_submodule(dep_name, |submod_ctx| {
            let module_res = ty::TyModule::type_check(submod_ctx, module);
            module_res.map(|module| ty::TySubmodule {
                library_name: library_name.clone(),
                module,
            })
        })
    }
}
