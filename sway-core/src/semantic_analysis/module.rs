use std::{
    collections::{HashMap, HashSet},
    fmt::Display,
    fs,
};

use graph_cycles::Cycles;
use sway_error::{
    error::CompileError,
    handler::{ErrorEmitted, Handler},
};
use sway_types::{BaseIdent, Named};

use crate::{
    decl_engine::DeclEngineGet,
    engine_threading::DebugWithEngines,
    language::{
        parsed::*,
        ty::{self, TyAstNodeContent, TyDecl},
        CallPath, ModName,
    },
    semantic_analysis::*,
    Engines, TypeInfo,
};

use super::declaration::auto_impl::AutoImplAbiEncodeContext;

#[derive(Clone, Debug)]
pub struct ModuleDepGraphEdge();

impl Display for ModuleDepGraphEdge {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "")
    }
}

pub type ModuleDepGraphNodeId = petgraph::graph::NodeIndex;

#[derive(Clone, Debug)]
pub enum ModuleDepGraphNode {
    Module {},
    Submodule { name: ModName },
}

impl DebugWithEngines for ModuleDepGraphNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>, _engines: &Engines) -> std::fmt::Result {
        let text = match self {
            ModuleDepGraphNode::Module { .. } => {
                format!("{:?}", "Root module")
            }
            ModuleDepGraphNode::Submodule { name: mod_name } => {
                format!("{:?}", mod_name.as_str())
            }
        };
        f.write_str(&text)
    }
}

// Represents an ordered graph between declaration id indexes.
pub type ModuleDepNodeGraph = petgraph::graph::DiGraph<ModuleDepGraphNode, ModuleDepGraphEdge>;

pub struct ModuleDepGraph {
    dep_graph: ModuleDepNodeGraph,
    root: ModuleDepGraphNodeId,
    node_name_map: HashMap<String, ModuleDepGraphNodeId>,
}

pub type ModuleEvaluationOrder = Vec<ModName>;

impl ModuleDepGraph {
    pub(crate) fn new() -> Self {
        Self {
            dep_graph: Default::default(),
            root: Default::default(),
            node_name_map: Default::default(),
        }
    }

    pub fn add_node(&mut self, node: ModuleDepGraphNode) -> ModuleDepGraphNodeId {
        let node_id = self.dep_graph.add_node(node.clone());
        match node {
            ModuleDepGraphNode::Module {} => {}
            ModuleDepGraphNode::Submodule { name: mod_name } => {
                self.node_name_map.insert(mod_name.to_string(), node_id);
            }
        };
        node_id
    }

    pub fn add_root_node(&mut self) -> ModuleDepGraphNodeId {
        self.root = self.add_node(super::module::ModuleDepGraphNode::Module {});
        self.root
    }

    fn get_node_id_for_module(
        &self,
        mod_name: &sway_types::BaseIdent,
    ) -> Option<ModuleDepGraphNodeId> {
        self.node_name_map.get(&mod_name.to_string()).copied()
    }

    /// Prints out GraphViz DOT format for the dependency graph.
    #[allow(dead_code)]
    pub(crate) fn visualize(&self, engines: &Engines, print_graph: Option<String>) {
        if let Some(graph_path) = print_graph {
            use petgraph::dot::{Config, Dot};
            let string_graph = self.dep_graph.filter_map(
                |_idx, node| Some(format!("{:?}", engines.help_out(node))),
                |_idx, edge| Some(format!("{}", edge)),
            );

            let output = format!(
                "{:?}",
                Dot::with_attr_getters(
                    &string_graph,
                    &[Config::NodeNoLabel, Config::EdgeNoLabel],
                    &|_, er| format!("label = {:?}", er.weight()),
                    &|_, nr| {
                        let _node = &self.dep_graph[nr.0];
                        let shape = "";
                        let url = "".to_string();
                        format!("{shape} label = {:?} {url}", nr.1)
                    },
                )
            );

            if graph_path.is_empty() {
                tracing::info!("{output}");
            } else {
                let result = fs::write(graph_path.clone(), output);
                if let Some(error) = result.err() {
                    tracing::error!(
                        "There was an issue while outputing module dep analysis graph to path {graph_path:?}\n{error}"
                    );
                }
            }
        }
    }

    /// Computes the ordered list by dependency, which will be used for evaluating the modules
    /// in the correct order. We run a topological sort and cycle finding algorithm to check
    /// for unsupported cyclic dependency cases.
    pub(crate) fn compute_order(
        &self,
        handler: &Handler,
    ) -> Result<ModuleEvaluationOrder, ErrorEmitted> {
        // Check for dependency cycles in the graph by running the Johnson's algorithm.
        let cycles = self.dep_graph.cycles();
        if !cycles.is_empty() {
            let mut modules = Vec::new();
            for cycle in cycles.first().unwrap() {
                let node = self.dep_graph.node_weight(*cycle).unwrap();
                match node {
                    ModuleDepGraphNode::Module {} => unreachable!(),
                    ModuleDepGraphNode::Submodule { name } => modules.push(name.clone()),
                };
            }
            return Err(handler.emit_err(CompileError::ModuleDepGraphCyclicReference { modules }));
        }

        // Do a topological sort to compute an ordered list of nodes.
        let sorted = match petgraph::algo::toposort(&self.dep_graph, None) {
            Ok(value) => value,
            // If we were not able to toposort, this means there is likely a cycle in the module dependency graph,
            // which we already handled above, so lets just return an empty evaluation order instead of panic'ing.
            // module dependencies, which we have already reported.
            Err(_) => return Err(handler.emit_err(CompileError::ModuleDepGraphEvaluationError {})),
        };

        let sorted = sorted
            .into_iter()
            .filter_map(|node_index| {
                let node = self.dep_graph.node_weight(node_index);
                match node {
                    Some(node) => match node {
                        ModuleDepGraphNode::Module {} => None, // root module
                        ModuleDepGraphNode::Submodule { name: mod_name } => Some(mod_name.clone()),
                    },
                    None => None,
                }
            })
            .rev()
            .collect::<Vec<_>>();

        Ok(sorted)
    }
}

impl ty::TyModule {
    /// Analyzes the given parsed module to produce a dependency graph.
    pub fn analyze(
        handler: &Handler,
        parsed: &ParseModule,
    ) -> Result<ModuleDepGraph, ErrorEmitted> {
        let mut dep_graph = ModuleDepGraph::new();
        dep_graph.add_root_node();

        let ParseModule { submodules, .. } = parsed;

        // Create graph nodes for each submodule.
        submodules.iter().for_each(|(name, _submodule)| {
            let sub_mod_node =
                dep_graph.add_node(ModuleDepGraphNode::Submodule { name: name.clone() });
            dep_graph
                .dep_graph
                .add_edge(dep_graph.root, sub_mod_node, ModuleDepGraphEdge {});
        });

        // Analyze submodules first in order of declaration.
        submodules.iter().for_each(|(name, submodule)| {
            let _ = ty::TySubmodule::analyze(handler, &mut dep_graph, name.clone(), submodule);
        });

        Ok(dep_graph)
    }

    /// Type-check the given parsed module to produce a typed module.
    ///
    /// Recursively type-checks submodules first.
    pub fn type_check(
        handler: &Handler,
        mut ctx: TypeCheckContext,
        parsed: &ParseModule,
        module_eval_order: ModuleEvaluationOrder,
    ) -> Result<Self, ErrorEmitted> {
        let ParseModule {
            submodules,
            tree,
            attributes,
            span,
            hash: _,
            ..
        } = parsed;

        // Type-check submodules first in order of evaluation previously computed by the dependency graph.
        let submodules_res = module_eval_order
            .iter()
            .map(|eval_mod_name| {
                let (name, submodule) = submodules
                    .iter()
                    .find(|(submod_name, _submodule)| eval_mod_name == submod_name)
                    .unwrap();
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

    // Filter and gather impl items
    fn get_all_impls(
        ctx: TypeCheckContext<'_>,
        nodes: &[AstNode],
        predicate: fn(&ImplTrait) -> bool,
    ) -> HashMap<BaseIdent, HashSet<CallPath>> {
        // Check which structs and enums needs to have auto impl for AbiEncode
        // We need to do this before type checking, because the impls must be right after
        // the declarations
        let mut impls = HashMap::<BaseIdent, HashSet<CallPath>>::new();

        for node in nodes.iter() {
            if let AstNodeContent::Declaration(Declaration::ImplTrait(decl)) = &node.content {
                let implementing_for = ctx.engines.te().get(decl.implementing_for.type_id);
                let implementing_for = match &*implementing_for {
                    TypeInfo::Struct(decl) => {
                        Some(ctx.engines().de().get(decl.id()).name().clone())
                    }
                    TypeInfo::Enum(decl) => Some(ctx.engines().de().get(decl.id()).name().clone()),
                    TypeInfo::Custom {
                        qualified_call_path,
                        ..
                    } => Some(qualified_call_path.call_path.suffix.clone()),
                    _ => None,
                };

                if let Some(implementing_for) = implementing_for {
                    if predicate(decl) {
                        impls
                            .entry(implementing_for)
                            .or_default()
                            .insert(decl.trait_name.clone());
                    }
                }
            }
        }

        impls
    }

    fn type_check_nodes(
        handler: &Handler,
        mut ctx: TypeCheckContext,
        nodes: Vec<AstNode>,
    ) -> Result<Vec<ty::TyAstNode>, ErrorEmitted> {
        let all_abiencode_impls = Self::get_all_impls(ctx.by_ref(), &nodes, |decl| {
            decl.trait_name.suffix.as_str() == "AbiEncode"
        });

        let mut typed_nodes = vec![];
        for node in nodes {
            let auto_impl_abiencode = match &node.content {
                AstNodeContent::Declaration(Declaration::StructDeclaration(decl)) => {
                    all_abiencode_impls.get(&decl.name).is_none()
                }
                AstNodeContent::Declaration(Declaration::EnumDeclaration(decl)) => {
                    all_abiencode_impls.get(&decl.name).is_none()
                }
                _ => false,
            };

            let Ok(node) = ty::TyAstNode::type_check(handler, ctx.by_ref(), node) else {
                continue;
            };

            let impl_node = match (auto_impl_abiencode, AutoImplAbiEncodeContext::new(&mut ctx)) {
                (true, Some(mut ctx)) => match &node.content {
                    TyAstNodeContent::Declaration(decl @ TyDecl::StructDecl(_))
                    | TyAstNodeContent::Declaration(decl @ TyDecl::EnumDecl(_)) => {
                        ctx.auto_impl_abi_encode(decl)
                    }
                    _ => None,
                },
                _ => None,
            };

            typed_nodes.push(node);
            typed_nodes.extend(impl_node);
        }

        Ok(typed_nodes)
    }
}

impl ty::TySubmodule {
    pub fn analyze(
        _handler: &Handler,
        module_dep_graph: &mut ModuleDepGraph,
        mod_name: ModName,
        submodule: &ParseSubmodule,
    ) -> Result<(), ErrorEmitted> {
        let ParseSubmodule { module, .. } = submodule;
        let sub_mod_node = module_dep_graph.get_node_id_for_module(&mod_name).unwrap();
        for node in module.tree.root_nodes.iter() {
            match &node.content {
                AstNodeContent::UseStatement(use_stmt) => {
                    if let Some(use_mod_ident) = use_stmt.call_path.first() {
                        if let Some(mod_name_node) =
                            module_dep_graph.get_node_id_for_module(use_mod_ident)
                        {
                            // Prevent adding edge loops between the same node as that will throw off
                            // the cyclic dependency analysis.
                            if sub_mod_node != mod_name_node {
                                module_dep_graph.dep_graph.add_edge(
                                    sub_mod_node,
                                    mod_name_node,
                                    ModuleDepGraphEdge {},
                                );
                            }
                        }
                    }
                }
                AstNodeContent::Declaration(_) => {}
                AstNodeContent::Expression(_) => {}
                AstNodeContent::ImplicitReturnExpression(_) => {}
                AstNodeContent::IncludeStatement(_) => {}
                AstNodeContent::Error(_, _) => {}
            }
        }
        Ok(())
    }

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
        let modules_dep_graph = ty::TyModule::analyze(handler, module)?;
        let module_eval_order = modules_dep_graph.compute_order(handler)?;
        parent_ctx.enter_submodule(mod_name, *visibility, module.span.clone(), |submod_ctx| {
            let module_res =
                ty::TyModule::type_check(handler, submod_ctx, module, module_eval_order);
            module_res.map(|module| ty::TySubmodule {
                module,
                mod_name_span: mod_name_span.clone(),
            })
        })
    }
}
