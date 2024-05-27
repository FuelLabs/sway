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
    decl_engine::{DeclEngineGet, DeclId},
    engine_threading::DebugWithEngines,
    language::{
        parsed::*,
        ty::{self, TyAstNodeContent, TyDecl},
        CallPath, ModName,
    },
    semantic_analysis::*,
    Engines, TypeInfo,
};

use super::{
    collection_context::SymbolCollectionContext,
    declaration::auto_impl::{self, EncodingAutoImplContext},
};

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
                        "There was an issue while outputting module dep analysis graph to path {graph_path:?}\n{error}"
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
    pub fn build_dep_graph(
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
            let _ =
                ty::TySubmodule::build_dep_graph(handler, &mut dep_graph, name.clone(), submodule);
        });

        Ok(dep_graph)
    }

    /// Collects the given parsed module to produce a module symbol map.
    ///
    /// Recursively collects submodules first.
    pub fn collect(
        handler: &Handler,
        engines: &Engines,
        ctx: &mut SymbolCollectionContext,
        parsed: &ParseModule,
    ) -> Result<(), ErrorEmitted> {
        let ParseModule {
            submodules,
            tree,
            module_eval_order,
            attributes: _,
            span: _,
            hash: _,
            ..
        } = parsed;

        // Analyze submodules first in order of evaluation previously computed by the dependency graph.
        module_eval_order.iter().for_each(|eval_mod_name| {
            let (name, submodule) = submodules
                .iter()
                .find(|(submod_name, _submodule)| eval_mod_name == submod_name)
                .unwrap();
            let _ = ty::TySubmodule::collect(handler, engines, ctx, name.clone(), submodule);
        });

        let _ = tree
            .root_nodes
            .iter()
            .map(|node| ty::TyAstNode::collect(handler, engines, ctx, node))
            .filter_map(|res| res.ok())
            .collect::<Vec<_>>();

        Ok(())
    }

    /// Type-check the given parsed module to produce a typed module.
    ///
    /// Recursively type-checks submodules first.
    pub fn type_check(
        handler: &Handler,
        mut ctx: TypeCheckContext,
        engines: &Engines,
        kind: TreeType,
        parsed: &ParseModule,
    ) -> Result<Self, ErrorEmitted> {
        let ParseModule {
            submodules,
            tree,
            attributes,
            span,
            module_eval_order,
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
                    ty::TySubmodule::type_check(
                        handler,
                        ctx.by_ref(),
                        engines,
                        name.clone(),
                        kind,
                        submodule,
                    )?,
                ))
            })
            .collect::<Result<Vec<_>, _>>();

        // TODO: Ordering should be solved across all modules prior to the beginning of type-check.
        let ordered_nodes = node_dependencies::order_ast_nodes_by_dependency(
            handler,
            ctx.engines(),
            tree.root_nodes.clone(),
        )?;

        let mut all_nodes = Self::type_check_nodes(handler, ctx.by_ref(), &ordered_nodes)?;

        let submodules = submodules_res?;

        let fallback_fn = collect_fallback_fn(&all_nodes, engines, handler)?;
        match (&kind, &fallback_fn) {
            (TreeType::Contract, _) | (_, None) => {}
            (_, Some(fallback_fn)) => {
                let fallback_fn = engines.de().get(fallback_fn);
                return Err(handler.emit_err(CompileError::FallbackFnsAreContractOnly {
                    span: fallback_fn.span.clone(),
                }));
            }
        }

        if ctx.experimental.new_encoding {
            let main_decl = all_nodes.iter_mut().find_map(|x| match &mut x.content {
                ty::TyAstNodeContent::Declaration(ty::TyDecl::FunctionDecl(decl)) => {
                    let fn_decl = engines.de().get_function(&decl.decl_id);
                    (fn_decl.name.as_str() == "main").then_some(fn_decl)
                }
                _ => None,
            });

            match (&kind, main_decl.is_some()) {
                (TreeType::Predicate, true) => {
                    let mut fn_generator =
                        auto_impl::EncodingAutoImplContext::new(&mut ctx).unwrap();
                    if let Ok(node) = fn_generator.generate_predicate_entry(
                        engines,
                        main_decl.as_ref().unwrap(),
                        handler,
                    ) {
                        all_nodes.push(node)
                    }
                }
                (TreeType::Script, true) => {
                    let mut fn_generator =
                        auto_impl::EncodingAutoImplContext::new(&mut ctx).unwrap();
                    if let Ok(node) = fn_generator.generate_script_entry(
                        engines,
                        main_decl.as_ref().unwrap(),
                        handler,
                    ) {
                        all_nodes.push(node)
                    }
                }
                (TreeType::Contract, _) => {
                    // collect all contract methods
                    let contract_fns = submodules
                        .iter()
                        .flat_map(|x| x.1.module.submodules_recursive())
                        .flat_map(|x| x.1.module.contract_fns(engines))
                        .chain(all_nodes.iter().flat_map(|x| x.contract_fns(engines)))
                        .collect::<Vec<_>>();

                    let mut fn_generator =
                        auto_impl::EncodingAutoImplContext::new(&mut ctx).unwrap();
                    if let Ok(node) = fn_generator.generate_contract_entry(
                        engines,
                        parsed.span.source_id().map(|x| x.program_id()),
                        &contract_fns,
                        fallback_fn,
                        handler,
                    ) {
                        all_nodes.push(node)
                    }
                }
                _ => {}
            }
        }

        Ok(Self {
            span: span.clone(),
            submodules,
            namespace: ctx.namespace.clone(),
            all_nodes,
            attributes: attributes.clone(),
        })
    }

    // Filter and gather impl items
    fn get_all_impls(
        ctx: TypeCheckContext<'_>,
        nodes: &[AstNode],
        predicate: fn(&ImplTrait) -> bool,
    ) -> HashMap<BaseIdent, HashSet<CallPath>> {
        let engines = ctx.engines();
        // Check which structs and enums needs to have auto impl for AbiEncode
        // We need to do this before type checking, because the impls must be right after
        // the declarations
        let mut impls = HashMap::<BaseIdent, HashSet<CallPath>>::new();

        for node in nodes.iter() {
            if let AstNodeContent::Declaration(Declaration::ImplTrait(decl_id)) = &node.content {
                let decl = &*engines.pe().get_impl_trait(decl_id);
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
        nodes: &[AstNode],
    ) -> Result<Vec<ty::TyAstNode>, ErrorEmitted> {
        let engines = ctx.engines();
        let all_abiencode_impls = Self::get_all_impls(ctx.by_ref(), nodes, |decl| {
            decl.trait_name.suffix.as_str() == "AbiEncode"
        });

        let mut typed_nodes = vec![];
        for node in nodes {
            let auto_impl_encoding_traits = match &node.content {
                AstNodeContent::Declaration(Declaration::StructDeclaration(decl_id)) => {
                    let decl = ctx.engines().pe().get_struct(decl_id);
                    all_abiencode_impls.get(&decl.name).is_none()
                }
                AstNodeContent::Declaration(Declaration::EnumDeclaration(decl_id)) => {
                    let decl = ctx.engines().pe().get_enum(decl_id);
                    all_abiencode_impls.get(&decl.name).is_none()
                }
                _ => false,
            };

            let Ok(node) = ty::TyAstNode::type_check(handler, ctx.by_ref(), node) else {
                continue;
            };

            if ctx.experimental.new_encoding {
                let mut generated = vec![];
                if let (true, Some(mut ctx)) = (
                    auto_impl_encoding_traits,
                    EncodingAutoImplContext::new(&mut ctx),
                ) {
                    match &node.content {
                        TyAstNodeContent::Declaration(decl @ TyDecl::StructDecl(_))
                        | TyAstNodeContent::Declaration(decl @ TyDecl::EnumDecl(_)) => {
                            let (a, b) = ctx.generate(engines, decl);
                            generated.extend(a);
                            generated.extend(b);
                        }
                        _ => {}
                    }
                };

                typed_nodes.push(node);
                typed_nodes.extend(generated);
            } else {
                typed_nodes.push(node);
            }
        }

        Ok(typed_nodes)
    }
}

fn collect_fallback_fn(
    all_nodes: &[ty::TyAstNode],
    engines: &Engines,
    handler: &Handler,
) -> Result<Option<DeclId<ty::TyFunctionDecl>>, ErrorEmitted> {
    let mut fallback_fns = all_nodes
        .iter()
        .filter_map(|x| match &x.content {
            ty::TyAstNodeContent::Declaration(ty::TyDecl::FunctionDecl(decl)) => {
                let d = engines.de().get(&decl.decl_id);
                d.is_fallback().then_some(decl.decl_id)
            }
            _ => None,
        })
        .collect::<Vec<_>>();

    let mut last_error = None;
    for f in fallback_fns.iter().skip(1) {
        let decl = engines.de().get(f);
        last_error = Some(
            handler.emit_err(CompileError::MultipleDefinitionsOfFallbackFunction {
                name: decl.name.clone(),
                span: decl.span.clone(),
            }),
        );
    }

    if let Some(last_error) = last_error {
        return Err(last_error);
    }

    if let Some(fallback_fn) = fallback_fns.pop() {
        let f = engines.de().get(&fallback_fn);
        if !f.parameters.is_empty() {
            Err(
                handler.emit_err(CompileError::FallbackFnsCannotHaveParameters {
                    span: f.span.clone(),
                }),
            )
        } else {
            Ok(Some(fallback_fn))
        }
    } else {
        Ok(None)
    }
}

impl ty::TySubmodule {
    pub fn build_dep_graph(
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
                AstNodeContent::IncludeStatement(_) => {}
                AstNodeContent::Error(_, _) => {}
            }
        }
        Ok(())
    }

    pub fn collect(
        handler: &Handler,
        engines: &Engines,
        parent_ctx: &mut SymbolCollectionContext,
        mod_name: ModName,
        submodule: &ParseSubmodule,
    ) -> Result<(), ErrorEmitted> {
        let ParseSubmodule {
            module,
            mod_name_span: _,
            visibility,
        } = submodule;
        parent_ctx.enter_submodule(
            engines,
            mod_name,
            *visibility,
            module.span.clone(),
            |submod_ctx| ty::TyModule::collect(handler, engines, submod_ctx, module),
        )
    }

    pub fn type_check(
        handler: &Handler,
        parent_ctx: TypeCheckContext,
        engines: &Engines,
        mod_name: ModName,
        kind: TreeType,
        submodule: &ParseSubmodule,
    ) -> Result<Self, ErrorEmitted> {
        let ParseSubmodule {
            module,
            mod_name_span,
            visibility,
        } = submodule;
        parent_ctx.enter_submodule(mod_name, *visibility, module.span.clone(), |submod_ctx| {
            let module_res = ty::TyModule::type_check(handler, submod_ctx, engines, kind, module);
            module_res.map(|module| ty::TySubmodule {
                module,
                mod_name_span: mod_name_span.clone(),
            })
        })
    }
}
