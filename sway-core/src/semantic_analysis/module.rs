use sway_error::handler::{ErrorEmitted, Handler};

use crate::{
    language::{parsed::*, ty, ModName},
    semantic_analysis::*,
};

impl ty::TyModule {
    /// Type-check the given parsed module to produce a typed module.
    ///
    /// Recursively type-checks submodules first.
    pub fn type_check(
        handler: &Handler,
        mut ctx: TypeCheckContext,
        parsed: &ParseModule,
    ) -> Result<Self, ErrorEmitted> {
        let ParseModule {
            submodules,
            tree,
            attributes,
            span,
            hash: _,
            ..
        } = parsed;

        // Type-check submodules first in order of declaration.
        let submodules_res = submodules
            .iter()
            .map(|(name, submodule)| {
                Ok((
                    name.clone(),
                    ty::TySubmodule::type_check(handler, ctx.by_ref(), name.clone(), submodule)?,
                ))
            })
            .collect::<Result<Vec<_>, _>>();

        // TODO: Ordering should be solved across all modules prior to the beginning of type-check.
        let ordered_nodes_res = node_dependencies::order_ast_nodes_by_dependency(
            handler,
            ctx.engines(),
            tree.root_nodes.clone(),
        );

        let typed_nodes_res = ordered_nodes_res
            .and_then(|ordered_nodes| Self::type_check_nodes(handler, ctx.by_ref(), ordered_nodes));

        submodules_res.and_then(|submodules| {
            typed_nodes_res.map(|all_nodes| Self {
                span: span.clone(),
                submodules,
                namespace: ctx.namespace.clone(),
                all_nodes,
                attributes: attributes.clone(),
            })
        })
    }

    fn type_check_nodes(
        handler: &Handler,
        mut ctx: TypeCheckContext,
        nodes: Vec<AstNode>,
    ) -> Result<Vec<ty::TyAstNode>, ErrorEmitted> {
        let typed_nodes = nodes
            .into_iter()
            .map(|node| ty::TyAstNode::type_check(handler, ctx.by_ref(), node))
            .filter_map(|res| res.ok())
            .collect();
        Ok(typed_nodes)
    }
}

impl ty::TySubmodule {
    pub fn type_check(
        handler: &Handler,
        parent_ctx: TypeCheckContext,
        mod_name: ModName,
        submodule: &ParseSubmodule,
    ) -> Result<Self, ErrorEmitted> {
        let ParseSubmodule {
            module,
            mod_name_span,
            visibility,
        } = submodule;
        parent_ctx.enter_submodule(mod_name, *visibility, module.span.clone(), |submod_ctx| {
            let module_res = ty::TyModule::type_check(handler, submod_ctx, module);
            module_res.map(|module| ty::TySubmodule {
                module,
                mod_name_span: mod_name_span.clone(),
            })
        })
    }
}
