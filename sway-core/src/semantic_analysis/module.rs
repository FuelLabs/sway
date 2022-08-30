use crate::{
    declaration_engine::declaration_engine::DeclarationEngine, error::*, parse_tree::*,
    semantic_analysis::*, type_system::*, types::ToCompileWrapper,
};

use sway_types::{Ident, Spanned};

#[derive(Clone, Debug)]
pub struct TypedModule {
    pub submodules: Vec<(DepName, TypedSubmodule)>,
    pub namespace: namespace::Module,
    pub all_nodes: Vec<TypedAstNode>,
}

#[derive(Clone, Debug)]
pub struct TypedSubmodule {
    pub library_name: Ident,
    pub module: TypedModule,
}

impl TypedModule {
    /// Type-check the given parsed module to produce a typed module.
    ///
    /// Recursively type-checks submodules first.
    pub fn type_check(mut ctx: TypeCheckContext, parsed: &ParseModule) -> CompileResult<Self> {
        let ParseModule { submodules, tree } = parsed;

        // Type-check submodules first in order of declaration.
        let mut submodules_res = ok(vec![], vec![], vec![]);
        for (name, submodule) in submodules {
            let submodule_res = TypedSubmodule::type_check(ctx.by_ref(), name.clone(), submodule);
            submodules_res = submodules_res.flat_map(|mut submodules| {
                submodule_res.map(|submodule| {
                    submodules.push((name.clone(), submodule));
                    submodules
                })
            });
        }

        // TODO: Ordering should be solved across all modules prior to the beginning of type-check.
        let ordered_nodes_res =
            node_dependencies::order_ast_nodes_by_dependency(tree.root_nodes.clone());

        let typed_nodes_res = ordered_nodes_res
            .flat_map(|ordered_nodes| Self::type_check_nodes(ctx.by_ref(), ordered_nodes));

        let validated_nodes_res = typed_nodes_res.flat_map(|typed_nodes| {
            let errors = check_supertraits(&typed_nodes, ctx.namespace, ctx.declaration_engine);
            ok(typed_nodes, vec![], errors)
        });

        submodules_res.flat_map(|submodules| {
            validated_nodes_res.map(|all_nodes| Self {
                submodules,
                namespace: ctx.namespace.module().clone(),
                all_nodes,
            })
        })
    }

    fn type_check_nodes(
        mut ctx: TypeCheckContext,
        nodes: Vec<AstNode>,
    ) -> CompileResult<Vec<TypedAstNode>> {
        let mut warnings = Vec::new();
        let mut errors = Vec::new();
        let typed_nodes = nodes
            .into_iter()
            .map(|node| TypedAstNode::type_check(ctx.by_ref(), node))
            .filter_map(|res| res.ok(&mut warnings, &mut errors))
            .collect();

        if !errors.is_empty() {
            err(warnings, errors)
        } else {
            ok(typed_nodes, warnings, errors)
        }
    }
}

impl TypedSubmodule {
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
            let module_res = TypedModule::type_check(submod_ctx, module);
            module_res.map(|module| TypedSubmodule {
                library_name: library_name.clone(),
                module,
            })
        })
    }
}

/// Given a list of typed AST nodes and a namespace, check whether all supertrait constraints are
/// satisfied. We're basically checking the following condition:
///    if trait B is implemented for type T, then trait A_i is also implemented for type T for
///    every A_i such that A_i is a supertrait of B.
///
/// This nicely works for transitive supertraits as well.
///
fn check_supertraits(
    typed_tree_nodes: &[TypedAstNode],
    namespace: &Namespace,
    declaration_engine: &DeclarationEngine,
) -> Vec<CompileError> {
    let mut errors = vec![];
    for node in typed_tree_nodes {
        if let TypedAstNodeContent::Declaration(TypedDeclaration::ImplTrait(TypedImplTrait {
            trait_name,
            span,
            implementing_for_type_id,
            ..
        })) = &node.content
        {
            if let CompileResult {
                value: Some(TypedDeclaration::TraitDeclaration(tr)),
                ..
            } = namespace.resolve_call_path(trait_name)
            {
                for supertrait in &tr.supertraits {
                    if !typed_tree_nodes.iter().any(|search_node| {
                        if let TypedAstNodeContent::Declaration(TypedDeclaration::ImplTrait(
                            TypedImplTrait {
                                trait_name: search_node_trait_name,
                                implementing_for_type_id: search_node_type_implementing_for,
                                ..
                            },
                        )) = &search_node.content
                        {
                            if let (
                                CompileResult {
                                    value: Some(TypedDeclaration::TraitDeclaration(tr1)),
                                    ..
                                },
                                CompileResult {
                                    value: Some(TypedDeclaration::TraitDeclaration(tr2)),
                                    ..
                                },
                            ) = (
                                namespace.resolve_call_path(search_node_trait_name),
                                namespace.resolve_call_path(&supertrait.name),
                            ) {
                                return (tr1.name == tr2.name)
                                    && (look_up_type_id(*implementing_for_type_id)
                                        .wrap_ref(declaration_engine)
                                        == look_up_type_id(*search_node_type_implementing_for)
                                            .wrap_ref(declaration_engine));
                            }
                        }
                        false
                    }) {
                        // The two errors below should really be a single error (and a "note"),
                        // but we don't have a way today to point to two separate locations in the
                        // user code with a single error.
                        errors.push(CompileError::SupertraitImplMissing {
                            supertrait_name: supertrait.name.clone(),
                            type_name: implementing_for_type_id.to_string(),
                            span: span.clone(),
                        });
                        errors.push(CompileError::SupertraitImplRequired {
                            supertrait_name: supertrait.name.clone(),
                            trait_name: tr.name.clone(),
                            span: tr.name.span().clone(),
                        });
                    }
                }
            }
        }
    }
    errors
}
