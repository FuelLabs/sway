use std::{
    collections::{HashMap, HashSet},
    fmt::Display,
    fs,
    sync::Arc,
};

use graph_cycles::Cycles;
use indexmap::IndexMap;
use itertools::Itertools;
use sway_error::{
    error::CompileError,
    handler::{ErrorEmitted, Handler},
    warning::{CompileWarning, Warning},
};
use sway_types::{BaseIdent, Named, SourceId, Span, Spanned};

use crate::{
    decl_engine::{DeclEngineGet, DeclId},
    engine_threading::{DebugWithEngines, PartialEqWithEngines, PartialEqWithEnginesContext},
    is_ty_module_cache_up_to_date,
    language::{
        parsed::*,
        ty::{self, TyAstNodeContent, TyDecl, TyEnumDecl},
        CallPath, ModName,
    },
    query_engine::{ModuleCacheKey, TypedModuleInfo},
    semantic_analysis::*,
    BuildConfig, Engines, TypeInfo,
};

use super::{
    declaration::auto_impl::{
        abi_encoding::AbiEncodingAutoImplContext, debug::DebugAutoImplContext,
        marker_traits::MarkerTraitsAutoImplContext,
    },
    symbol_collection_context::SymbolCollectionContext,
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
                |_idx, edge| Some(format!("{edge}")),
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

    /// Retrieves a cached typed module if it's up to date.
    ///
    /// This function checks the cache for a typed module corresponding to the given source ID.
    /// If found and up to date, it returns the cached module. Otherwise, it returns None.
    fn get_cached_ty_module_if_up_to_date(
        source_id: Option<&SourceId>,
        engines: &Engines,
        build_config: Option<&BuildConfig>,
    ) -> Option<(Arc<ty::TyModule>, Arc<namespace::Module>)> {
        let source_id = source_id?;

        // Create a cache key and get the module cache
        let path = engines.se().get_path(source_id);
        let include_tests = build_config.is_some_and(|x| x.include_tests);
        let key = ModuleCacheKey::new(path.clone().into(), include_tests);
        let cache = engines.qe().module_cache.read();
        cache.get(&key).and_then(|entry| {
            entry.typed.as_ref().and_then(|typed| {
                // Check if the cached module is up to date
                let is_up_to_date = is_ty_module_cache_up_to_date(
                    engines,
                    &path.into(),
                    include_tests,
                    build_config,
                );

                // Return the cached module if it's up to date, otherwise None
                if is_up_to_date {
                    Some((typed.module.clone(), typed.namespace_module.clone()))
                } else {
                    None
                }
            })
        })
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
        build_config: Option<&BuildConfig>,
    ) -> Result<Arc<Self>, ErrorEmitted> {
        let ParseModule {
            submodules,
            tree,
            attributes,
            span,
            module_eval_order,
            ..
        } = parsed;

        // Try to get the cached root module if it's up to date
        if let Some((ty_module, _namespace_module)) =
            ty::TyModule::get_cached_ty_module_if_up_to_date(
                parsed.span.source_id(),
                engines,
                build_config,
            )
        {
            return Ok(ty_module);
        }

        // Type-check submodules first in order of evaluation previously computed by the dependency graph.
        let submodules_res = module_eval_order
            .iter()
            .map(|eval_mod_name| {
                let (name, submodule) = submodules
                    .iter()
                    .find(|(submod_name, _)| eval_mod_name == submod_name)
                    .unwrap();

                // Try to get the cached submodule
                if let Some(cached_module) = ty::TyModule::get_cached_ty_module_if_up_to_date(
                    submodule.module.span.source_id(),
                    engines,
                    build_config,
                ) {
                    // If cached, restore namespace module and return cached TySubmodule
                    let (ty_module, namespace_module) = cached_module;
                    ctx.namespace_mut()
                        .current_module_mut()
                        .import_cached_submodule(name, (*namespace_module).clone());

                    let ty_submod = ty::TySubmodule {
                        module: ty_module,
                        mod_name_span: submodule.mod_name_span.clone(),
                    };
                    Ok::<(BaseIdent, ty::TySubmodule), ErrorEmitted>((name.clone(), ty_submod))
                } else {
                    // If not cached, type-check the submodule
                    let type_checked_submodule = ty::TySubmodule::type_check(
                        handler,
                        ctx.by_ref(),
                        engines,
                        name.clone(),
                        kind,
                        submodule,
                        build_config,
                    )?;
                    Ok((name.clone(), type_checked_submodule))
                }
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
                    let mut fn_generator = AbiEncodingAutoImplContext::new(&mut ctx);
                    if let Ok(node) = fn_generator.generate_predicate_entry(
                        engines,
                        main_decl.as_ref().unwrap(),
                        handler,
                    ) {
                        all_nodes.push(node)
                    }
                }
                (TreeType::Script, true) => {
                    let mut fn_generator = AbiEncodingAutoImplContext::new(&mut ctx);
                    if let Ok(node) = fn_generator.generate_script_entry(
                        engines,
                        main_decl.as_ref().unwrap(),
                        handler,
                    ) {
                        all_nodes.push(node)
                    }
                }
                (TreeType::Contract, _) => {
                    // collect all supertrait methods
                    let contract_supertrait_fns = submodules
                        .iter()
                        .flat_map(|x| x.1.module.submodules_recursive())
                        .flat_map(|x| x.1.module.contract_supertrait_fns(engines))
                        .chain(
                            all_nodes
                                .iter()
                                .flat_map(|x| x.contract_supertrait_fns(engines)),
                        )
                        .collect::<Vec<_>>();

                    // collect all contract methods
                    let mut contract_fns = submodules
                        .iter()
                        .flat_map(|x| x.1.module.submodules_recursive())
                        .flat_map(|x| x.1.module.contract_fns(engines))
                        .chain(all_nodes.iter().flat_map(|x| x.contract_fns(engines)))
                        .collect::<Vec<_>>();

                    // exclude all contract methods that are supertrait methods
                    let partialeq_ctx = PartialEqWithEnginesContext::new(engines);
                    contract_fns.retain(|method| {
                        contract_supertrait_fns
                            .iter()
                            .all(|si| !PartialEqWithEngines::eq(method, si, &partialeq_ctx))
                    });

                    let mut fn_generator = AbiEncodingAutoImplContext::new(&mut ctx);
                    if let Ok(node) = fn_generator.generate_contract_entry(
                        engines,
                        parsed.span.source_id(),
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

        #[allow(clippy::arc_with_non_send_sync)]
        let ty_module = Arc::new(Self {
            span: span.clone(),
            submodules,
            all_nodes,
            attributes: attributes.clone(),
        });

        // Cache the ty module
        if let Some(source_id) = span.source_id() {
            let path = engines.se().get_path(source_id);
            let version = build_config
                .and_then(|config| config.lsp_mode.as_ref())
                .and_then(|lsp| lsp.file_versions.get(&path).copied())
                .flatten();

            let include_tests = build_config.is_some_and(|x| x.include_tests);
            let key = ModuleCacheKey::new(path.clone().into(), include_tests);
            engines.qe().update_typed_module_cache_entry(
                &key,
                TypedModuleInfo {
                    module: ty_module.clone(),
                    namespace_module: Arc::new(ctx.namespace().current_module().clone()),
                    version,
                },
            );
        }

        Ok(ty_module)
    }

    // Filter and gather impl items
    fn get_all_impls(
        ctx: TypeCheckContext<'_>,
        nodes: &[AstNode],
        predicate: fn(&crate::parsed::ImplSelfOrTrait) -> bool,
    ) -> HashMap<BaseIdent, HashSet<CallPath>> {
        let engines = ctx.engines();
        let mut impls = HashMap::<BaseIdent, HashSet<CallPath>>::new();

        for node in nodes.iter() {
            if let AstNodeContent::Declaration(Declaration::ImplSelfOrTrait(decl_id)) =
                &node.content
            {
                let decl = &*engines.pe().get_impl_self_or_trait(decl_id);
                let implementing_for = ctx.engines.te().get(decl.implementing_for.type_id);
                let implementing_for = match &*implementing_for {
                    TypeInfo::Struct(decl_id) => {
                        Some(ctx.engines().de().get(decl_id).name().clone())
                    }
                    TypeInfo::Enum(decl) => Some(ctx.engines().de().get(decl).name().clone()),
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

        // Check which structs and enums needs to have auto impl for `AbiEncode` and `AbiDecode`.
        // We need to do this before type checking, because the impls must be right after
        // the declarations.
        let all_abi_encode_impls = Self::get_all_impls(ctx.by_ref(), nodes, |decl| {
            decl.trait_name.suffix.as_str() == "AbiEncode"
        });
        let all_debug_impls = Self::get_all_impls(ctx.by_ref(), nodes, |decl| {
            decl.trait_name.suffix.as_str() == "Debug"
        });

        let mut typed_nodes = vec![];
        for node in nodes {
            // Check if the encoding and debug traits are explicitly implemented.
            let (auto_impl_encoding_traits, auto_impl_debug_traits) = match &node.content {
                AstNodeContent::Declaration(Declaration::StructDeclaration(decl_id)) => {
                    let decl = ctx.engines().pe().get_struct(decl_id);
                    (
                        !all_abi_encode_impls.contains_key(&decl.name),
                        !all_debug_impls.contains_key(&decl.name),
                    )
                }
                AstNodeContent::Declaration(Declaration::EnumDeclaration(decl_id)) => {
                    let decl = ctx.engines().pe().get_enum(decl_id);
                    (
                        !all_abi_encode_impls.contains_key(&decl.name),
                        !all_debug_impls.contains_key(&decl.name),
                    )
                }
                _ => (false, false),
            };

            let Ok(node) = ty::TyAstNode::type_check(handler, ctx.by_ref(), node) else {
                continue;
            };

            // Auto impl encoding traits only if they are not explicitly implemented.
            let mut generated = vec![];
            if ctx.experimental.new_encoding {
                if let (true, mut ctx) = (
                    auto_impl_encoding_traits,
                    AbiEncodingAutoImplContext::new(&mut ctx),
                ) {
                    match &node.content {
                        TyAstNodeContent::Declaration(decl @ TyDecl::StructDecl(_))
                        | TyAstNodeContent::Declaration(decl @ TyDecl::EnumDecl(_)) => {
                            let (abi_encode_impl, abi_decode_impl) =
                                ctx.generate_abi_encode_and_decode_impls(engines, decl);
                            generated.extend(abi_encode_impl);
                            generated.extend(abi_decode_impl);
                        }
                        _ => {}
                    }
                };
            }

            // Auto impl debug traits only if they are not explicitly implemented
            if auto_impl_debug_traits {
                match &node.content {
                    TyAstNodeContent::Declaration(decl @ TyDecl::StructDecl(_))
                    | TyAstNodeContent::Declaration(decl @ TyDecl::EnumDecl(_)) => {
                        let mut ctx = DebugAutoImplContext::new(&mut ctx);
                        let a = ctx.generate_debug_impl(engines, decl);
                        generated.extend(a);
                    }
                    _ => {}
                }
            }

            // Always auto impl marker traits. If an explicit implementation exists, that will be
            // reported as an error when type-checking trait impls.
            let mut ctx = MarkerTraitsAutoImplContext::new(&mut ctx);
            if let TyAstNodeContent::Declaration(TyDecl::EnumDecl(enum_decl)) = &node.content {
                let enum_decl = &*ctx.engines().de().get(&enum_decl.decl_id);

                let enum_marker_trait_impl =
                    ctx.generate_enum_marker_trait_impl(engines, enum_decl);
                generated.extend(enum_marker_trait_impl);

                if check_is_valid_error_type_enum(handler, enum_decl).is_ok_and(|res| res) {
                    let error_type_marker_trait_impl =
                        ctx.generate_error_type_marker_trait_impl_for_enum(engines, enum_decl);
                    generated.extend(error_type_marker_trait_impl);
                }
            }

            typed_nodes.push(node);
            typed_nodes.extend(generated);
        }

        Ok(typed_nodes)
    }
}

/// Performs all semantic checks for `error_type` and `error` attributes, and returns true if the
/// `enum_decl` is a valid error type declaration.
fn check_is_valid_error_type_enum(
    handler: &Handler,
    enum_decl: &TyEnumDecl,
) -> Result<bool, ErrorEmitted> {
    let has_error_type_attribute = enum_decl.attributes.has_error_type();

    if has_error_type_attribute && enum_decl.variants.is_empty() {
        handler.emit_warn(CompileWarning {
            span: enum_decl.name().span(),
            warning_content: Warning::ErrorTypeEmptyEnum {
                enum_name: enum_decl.name().into(),
            },
        });
    }

    // We show warnings for error messages even if the error type enum
    // is not well formed, e.g., if it doesn't have the `error_type` attribute.
    let mut duplicated_error_messages = IndexMap::<&str, Vec<Span>>::new();
    for (enum_variant_name, error_attr) in enum_decl.variants.iter().flat_map(|variant| {
        variant
            .attributes
            .error()
            .map(|error_attr| (&variant.name, error_attr))
    }) {
        error_attr.check_args_multiplicity(handler)?;
        assert_eq!(
            (1usize, 1usize),
            (&error_attr.args_multiplicity()).into(),
            "`#[error]` attribute must have argument multiplicity of exactly one"
        );

        let m_arg = &error_attr.args[0];
        let error_msg = m_arg.get_string(handler, error_attr)?;

        if error_msg.is_empty() {
            handler.emit_warn(CompileWarning {
                span: m_arg
                    .value
                    .as_ref()
                    .expect("`m` argument has a valid empty string value")
                    .span(),
                warning_content: Warning::ErrorEmptyErrorMessage {
                    enum_name: enum_decl.name().clone(),
                    enum_variant_name: enum_variant_name.clone(),
                },
            });
        } else {
            // We ignore duplicated empty messages and for those show
            // only the warning that the message is empty.
            duplicated_error_messages
                .entry(error_msg)
                .or_default()
                .push(
                    m_arg
                        .value
                        .as_ref()
                        .expect("`m` argument has a valid empty string value")
                        .span(),
                );
        }
    }

    // Emit duplicated messages warnings, if we actually have duplicates.
    for duplicated_error_messages in duplicated_error_messages
        .into_values()
        .filter(|spans| spans.len() > 1)
    {
        let (last_occurrence, previous_occurrences) = duplicated_error_messages
            .split_last()
            .expect("`duplicated_error_messages` has more than one element");
        handler.emit_warn(CompileWarning {
            span: last_occurrence.clone(),
            warning_content: Warning::ErrorDuplicatedErrorMessage {
                last_occurrence: last_occurrence.clone(),
                previous_occurrences: previous_occurrences.into(),
            },
        });
    }

    handler.scope(|handler| {
        if has_error_type_attribute {
            let non_error_variants = enum_decl
                .variants
                .iter()
                .filter(|variant| !variant.attributes.has_error())
                .collect_vec();
            if !non_error_variants.is_empty() {
                handler.emit_err(CompileError::ErrorTypeEnumHasNonErrorVariants {
                    enum_name: enum_decl.name().into(),
                    non_error_variants: non_error_variants
                        .iter()
                        .map(|variant| (&variant.name).into())
                        .collect(),
                });
            }
        } else {
            for variant in enum_decl
                .variants
                .iter()
                .filter(|variant| variant.attributes.has_error())
            {
                handler.emit_err(CompileError::ErrorAttributeInNonErrorEnum {
                    enum_name: enum_decl.name().into(),
                    enum_variant_name: (&variant.name).into(),
                });
            }
        }

        Ok(())
    })?;

    Ok(has_error_type_attribute)
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
            handler,
            engines,
            mod_name,
            *visibility,
            module.span.clone(),
            |submod_ctx| ty::TyModule::collect(handler, engines, submod_ctx, module),
        )?
    }

    pub fn type_check(
        handler: &Handler,
        mut parent_ctx: TypeCheckContext,
        engines: &Engines,
        mod_name: ModName,
        kind: TreeType,
        submodule: &ParseSubmodule,
        build_config: Option<&BuildConfig>,
    ) -> Result<Self, ErrorEmitted> {
        let ParseSubmodule {
            module,
            mod_name_span,
            visibility,
        } = submodule;
        parent_ctx.enter_submodule(
            handler,
            mod_name,
            *visibility,
            module.span.clone(),
            |submod_ctx| {
                let module_res = ty::TyModule::type_check(
                    handler,
                    submod_ctx,
                    engines,
                    kind,
                    module,
                    build_config,
                );
                module_res.map(|module| ty::TySubmodule {
                    module,
                    mod_name_span: mod_name_span.clone(),
                })
            },
        )?
    }
}
