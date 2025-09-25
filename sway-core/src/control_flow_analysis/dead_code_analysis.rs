use super::*;
use crate::{
    decl_engine::*,
    language::{
        parsed::TreeType,
        ty::{
            self, ConfigurableDecl, ConstantDecl, FunctionDecl, ProjectionKind, StructDecl,
            TraitDecl, TyAstNode, TyAstNodeContent, TyDecl, TyImplItem, TypeAliasDecl,
        },
        CallPath, CallPathType, Visibility,
    },
    transform::Attributes,
    type_system::TypeInfo,
    Engines, GenericArgument, TypeEngine, TypeId,
};
use petgraph::{prelude::NodeIndex, visit::Dfs};
use std::collections::{BTreeSet, HashMap};
use sway_ast::Intrinsic;
use sway_error::{error::CompileError, type_error::TypeError};
use sway_error::{
    handler::Handler,
    warning::{CompileWarning, Warning},
};
use sway_types::{constants::STD, span::Span, Ident, Named, Spanned};

// Defines if this node is a root in the dca graph or not
fn is_entry_point(node: &TyAstNode, decl_engine: &DeclEngine, tree_type: &TreeType) -> bool {
    match tree_type {
        TreeType::Predicate | TreeType::Script => {
            // Predicates and scripts have main and test functions as entry points.
            match node {
                TyAstNode {
                    span: _,
                    content:
                        TyAstNodeContent::Declaration(TyDecl::FunctionDecl(FunctionDecl {
                            decl_id,
                            ..
                        })),
                    ..
                } => {
                    let decl = decl_engine.get_function(decl_id);
                    decl.is_entry() || decl.is_main() || decl.is_test()
                }
                _ => false,
            }
        }
        TreeType::Contract | TreeType::Library => match node {
            TyAstNode {
                content:
                    TyAstNodeContent::Declaration(TyDecl::FunctionDecl(FunctionDecl { decl_id })),
                ..
            } => {
                let decl = decl_engine.get_function(decl_id);
                decl.visibility == Visibility::Public || decl.is_test() || decl.is_fallback()
            }
            TyAstNode {
                content: TyAstNodeContent::Declaration(TyDecl::TraitDecl(TraitDecl { decl_id })),
                ..
            } => decl_engine.get_trait(decl_id).visibility.is_public(),
            TyAstNode {
                content:
                    TyAstNodeContent::Declaration(TyDecl::StructDecl(StructDecl { decl_id, .. })),
                ..
            } => {
                let struct_decl = decl_engine.get_struct(decl_id);
                struct_decl.visibility == Visibility::Public
            }
            TyAstNode {
                content: TyAstNodeContent::Declaration(TyDecl::ImplSelfOrTrait { .. }),
                ..
            } => true,
            TyAstNode {
                content:
                    TyAstNodeContent::Declaration(TyDecl::ConstantDecl(ConstantDecl { decl_id })),
                ..
            } => {
                let decl = decl_engine.get_constant(decl_id);
                decl.visibility.is_public()
            }
            TyAstNode {
                content:
                    TyAstNodeContent::Declaration(TyDecl::ConfigurableDecl(ConfigurableDecl { .. })),
                ..
            } => false,
            TyAstNode {
                content:
                    TyAstNodeContent::Declaration(TyDecl::TypeAliasDecl(TypeAliasDecl {
                        decl_id, ..
                    })),
                ..
            } => {
                let decl = decl_engine.get_type_alias(decl_id);
                decl.visibility.is_public()
            }
            _ => false,
        },
    }
}

impl<'cfg> ControlFlowGraph<'cfg> {
    pub(crate) fn find_dead_code(&self, decl_engine: &DeclEngine) -> Vec<CompileWarning> {
        // Dead code is code that has no path from the entry point.
        // Collect all connected nodes by traversing from the entries.
        // The dead nodes are those we did not collect.
        let mut connected_from_entry = BTreeSet::new();
        let mut dfs = Dfs::empty(&self.graph);
        for &entry in &self.entry_points {
            dfs.move_to(entry);
            while let Some(node) = dfs.next(&self.graph) {
                connected_from_entry.insert(node);
            }
        }

        // Collect all nodes that are connected from another node.
        let mut connections_count: HashMap<NodeIndex, u32> = HashMap::<NodeIndex, u32>::new();
        for edge in self.graph.raw_edges() {
            if let Some(count) = connections_count.get(&edge.target()) {
                connections_count.insert(edge.target(), count + 1);
            } else {
                connections_count.insert(edge.target(), 1);
            }
        }

        let is_dead_check = |n: &NodeIndex| {
            match &self.graph[*n] {
                ControlFlowGraphNode::ProgramNode {
                    node:
                        ty::TyAstNode {
                            content:
                                ty::TyAstNodeContent::Declaration(ty::TyDecl::VariableDecl { .. }),
                            ..
                        },
                    ..
                } => {
                    // Consider variables declarations dead when count is not greater than 1
                    connections_count
                        .get(n)
                        .cloned()
                        .is_none_or(|count| count <= 1)
                }
                ControlFlowGraphNode::FunctionParameter { .. } => {
                    // Consider variables declarations dead when count is not greater than 1
                    // Function param always has the function pointing to them
                    connections_count
                        .get(n)
                        .cloned()
                        .is_none_or(|count| count <= 1)
                }
                _ => false,
            }
        };

        let is_alive_check = |n: &NodeIndex| {
            match &self.graph[*n] {
                ControlFlowGraphNode::ProgramNode {
                    node:
                        ty::TyAstNode {
                            content:
                                ty::TyAstNodeContent::Declaration(ty::TyDecl::VariableDecl(decl)),
                            ..
                        },
                    ..
                } => {
                    if decl.name.as_str().starts_with('_') {
                        true
                    } else {
                        // Consider variables declarations alive when count is greater than 1
                        // This is explicitly required because the variable may be considered dead
                        // when it is not connected from an entry point, while it may still be used by other dead code.
                        connections_count
                            .get(n)
                            .cloned()
                            .is_some_and(|count| count > 1)
                    }
                }
                ControlFlowGraphNode::FunctionParameter {
                    param_name,
                    is_self,
                    ..
                } => {
                    if *is_self || param_name.as_str().starts_with('_') {
                        // self type parameter is always alive
                        true
                    } else {
                        // Consider param alive when count is greater than 1
                        // This is explicitly required because the param may be considered dead
                        // when it is not connected from an entry point, while it may still be used by other dead code.
                        connections_count
                            .get(n)
                            .cloned()
                            .is_none_or(|count| count > 1)
                    }
                }
                ControlFlowGraphNode::ProgramNode {
                    node:
                        ty::TyAstNode {
                            content:
                                ty::TyAstNodeContent::Declaration(ty::TyDecl::ImplSelfOrTrait {
                                    ..
                                }),
                            ..
                        },
                    ..
                } => {
                    // Consider impls always alive.
                    // Consider it alive when it does not have any methods.
                    // Also consider it alive when it contains unused methods inside.
                    true
                }
                ControlFlowGraphNode::StructField { .. } => {
                    // Consider struct field alive when count is greater than 0
                    connections_count
                        .get(n)
                        .cloned()
                        .is_some_and(|count| count > 0)
                }
                _ => false,
            }
        };

        let dead_nodes: Vec<_> = self
            .graph
            .node_indices()
            .filter(|n| {
                (!connected_from_entry.contains(n) || is_dead_check(n)) && !is_alive_check(n)
            })
            .collect();

        let dead_function_contains_span = |span: &Span| -> bool {
            dead_nodes.iter().any(|x| {
                if let ControlFlowGraphNode::ProgramNode {
                    node:
                        ty::TyAstNode {
                            span: function_span,
                            content:
                                ty::TyAstNodeContent::Declaration(ty::TyDecl::FunctionDecl { .. }),
                        },
                    ..
                } = &self.graph[*x]
                {
                    function_span.end() >= span.end() && function_span.start() <= span.start()
                } else {
                    false
                }
            })
        };

        let priv_enum_var_warn = |name: &Ident| CompileWarning {
            span: name.span(),
            warning_content: Warning::DeadEnumVariant {
                variant_name: name.clone(),
            },
        };
        let dead_enum_variant_warnings = dead_nodes
            .iter()
            .filter_map(|x| {
                // If dead code is allowed return immediately no warning.
                if allow_dead_code_node(decl_engine, &self.graph, &self.graph[*x]) {
                    None
                } else {
                    match &self.graph[*x] {
                        ControlFlowGraphNode::EnumVariant {
                            variant_name,
                            is_public,
                            ..
                        } if !is_public => Some(priv_enum_var_warn(variant_name)),
                        _ => None,
                    }
                }
            })
            .collect::<Vec<_>>();

        let dead_ast_node_warnings = dead_nodes
            .iter()
            .filter_map(|x| {
                // If dead code is allowed return immediately no warning.
                if allow_dead_code_node(decl_engine, &self.graph, &self.graph[*x]) {
                    None
                } else {
                    match &self.graph[*x] {
                        ControlFlowGraphNode::ProgramNode { node, .. } => {
                            construct_dead_code_warning_from_node(decl_engine, node)
                        }
                        ControlFlowGraphNode::EnumVariant {
                            variant_name,
                            is_public,
                            ..
                        } if !is_public => Some(priv_enum_var_warn(variant_name)),
                        ControlFlowGraphNode::EnumVariant { .. } => None,
                        ControlFlowGraphNode::MethodDeclaration { span, .. } => {
                            Some(CompileWarning {
                                span: span.clone(),
                                warning_content: Warning::DeadMethod,
                            })
                        }
                        ControlFlowGraphNode::StructField {
                            struct_field_name, ..
                        } => Some(CompileWarning {
                            span: struct_field_name.span(),
                            warning_content: Warning::StructFieldNeverRead,
                        }),
                        ControlFlowGraphNode::StorageField { field_name, .. } => {
                            Some(CompileWarning {
                                span: field_name.span(),
                                warning_content: Warning::DeadStorageDeclaration,
                            })
                        }
                        ControlFlowGraphNode::OrganizationalDominator(..) => None,
                        ControlFlowGraphNode::FunctionParameter { param_name, .. } => {
                            Some(CompileWarning {
                                span: param_name.span(),
                                warning_content: Warning::DeadDeclaration,
                            })
                        }
                    }
                }
            })
            .collect::<Vec<_>>();

        let all_warnings = [dead_enum_variant_warnings, dead_ast_node_warnings].concat();
        // filter out any overlapping spans -- if a span is contained within another one,
        // remove it.
        all_warnings
            .clone()
            .into_iter()
            .filter(
                |CompileWarning {
                     span,
                     warning_content,
                 }| {
                    if let Warning::UnreachableCode = warning_content {
                        // If the unreachable code is within an unused function, filter it out
                        // since the dead function name is the only warning we want to show.
                        if dead_function_contains_span(span) {
                            return false;
                        }
                    }

                    // if any other warnings contain a span which completely covers this one, filter
                    // out this one.
                    !all_warnings.iter().any(
                        |CompileWarning {
                             span: other_span, ..
                         }| {
                            other_span.end() > span.end() && other_span.start() < span.start()
                        },
                    )
                },
            )
            .collect()
    }

    pub(crate) fn append_module_to_dead_code_graph<'eng: 'cfg>(
        engines: &'eng Engines,
        module_nodes: &[ty::TyAstNode],
        tree_type: &TreeType,
        graph: &mut ControlFlowGraph<'cfg>,
        // the `Result` return is just to handle `Unimplemented` errors
    ) -> Result<(), CompileError> {
        // do a depth first traversal and cover individual inner ast nodes
        let decl_engine = engines.de();
        let exit_node = Some(graph.add_node(("Program exit".to_string()).into()));

        let mut entry_points = vec![];
        let mut non_entry_points = vec![];

        for ast_node in module_nodes {
            if is_entry_point(ast_node, decl_engine, tree_type) {
                entry_points.push(ast_node);
            } else {
                non_entry_points.push(ast_node);
            }
        }

        for ast_entrypoint in non_entry_points.into_iter().chain(entry_points) {
            let (_l_leaves, _new_exit_node) = connect_node(
                engines,
                ast_entrypoint,
                graph,
                &[],
                exit_node,
                tree_type,
                NodeConnectionOptions::default(),
            )?;
        }
        graph.entry_points = collect_entry_points(decl_engine, tree_type, &graph.graph)?;
        Ok(())
    }
}

/// Collect all entry points into the graph based on the tree type.
fn collect_entry_points(
    decl_engine: &DeclEngine,
    tree_type: &TreeType,
    graph: &flow_graph::Graph,
) -> Result<Vec<flow_graph::EntryPoint>, CompileError> {
    let mut entry_points = vec![];
    for i in graph.node_indices() {
        let is_entry = match &graph[i] {
            ControlFlowGraphNode::ProgramNode { node, .. } => {
                is_entry_point(node, decl_engine, tree_type)
            }
            _ => false,
        };
        if is_entry {
            entry_points.push(i);
        }
    }

    Ok(entry_points)
}

/// This struct is used to pass node connection further down the tree as
/// we are processing AST nodes.
#[derive(Clone, Copy, Default)]
struct NodeConnectionOptions {
    /// When this is enabled, connect struct fields to the struct itself,
    /// thus making all struct fields considered as being used in the graph.
    force_struct_fields_connection: bool,
    parent_node: Option<NodeIndex>,
}

fn connect_node<'eng: 'cfg, 'cfg>(
    engines: &'eng Engines,
    node: &ty::TyAstNode,
    graph: &mut ControlFlowGraph<'cfg>,
    leaves: &[NodeIndex],
    exit_node: Option<NodeIndex>,
    tree_type: &TreeType,
    options: NodeConnectionOptions,
) -> Result<(Vec<NodeIndex>, Option<NodeIndex>), CompileError> {
    //    let mut graph = graph.clone();
    let span = node.span.clone();
    Ok(match &node.content {
        ty::TyAstNodeContent::Expression(ty::TyExpression {
            expression: expr_variant,
            span,
            ..
        }) => {
            let entry = graph.add_node(ControlFlowGraphNode::from_node_with_parent(
                node,
                options.parent_node,
            ));
            // insert organizational dominator node
            // connected to all current leaves
            for leaf in leaves {
                graph.add_edge(*leaf, entry, "".into());
            }

            (
                connect_expression(
                    engines,
                    expr_variant,
                    graph,
                    &[entry],
                    exit_node,
                    "",
                    tree_type,
                    span.clone(),
                    options,
                )?,
                match expr_variant {
                    ty::TyExpressionVariant::ImplicitReturn(_) => None,
                    _ => exit_node,
                },
            )
        }
        ty::TyAstNodeContent::SideEffect(_) => (leaves.to_vec(), exit_node),
        ty::TyAstNodeContent::Declaration(decl) => {
            // all leaves connect to this node, then this node is the singular leaf
            let cfg_node: ControlFlowGraphNode =
                ControlFlowGraphNode::from_node_with_parent(node, options.parent_node);
            // check if node for this decl already exists
            let decl_node = match graph.get_node_from_decl(&cfg_node) {
                Some(node) => node,
                None => graph.add_node(cfg_node),
            };
            for leaf in leaves {
                graph.add_edge(*leaf, decl_node, "".into());
            }
            (
                connect_declaration(
                    engines, decl, graph, decl_node, span, exit_node, tree_type, leaves, options,
                )?,
                exit_node,
            )
        }
        ty::TyAstNodeContent::Error(_, _) => (vec![], None),
    })
}

#[allow(clippy::too_many_arguments)]
fn connect_declaration<'eng: 'cfg, 'cfg>(
    engines: &'eng Engines,
    decl: &ty::TyDecl,
    graph: &mut ControlFlowGraph<'cfg>,
    entry_node: NodeIndex,
    span: Span,
    exit_node: Option<NodeIndex>,
    tree_type: &TreeType,
    leaves: &[NodeIndex],
    options: NodeConnectionOptions,
) -> Result<Vec<NodeIndex>, CompileError> {
    let decl_engine = engines.de();
    match decl {
        ty::TyDecl::VariableDecl(var_decl) => {
            let ty::TyVariableDecl {
                body,
                name,
                type_ascription,
                ..
            } = &**var_decl;

            // Connect variable declaration node to body expression.
            let result = connect_expression(
                engines,
                &body.expression,
                graph,
                &[entry_node],
                exit_node,
                "variable instantiation",
                tree_type,
                body.clone().span,
                options,
            );

            if let Ok(ref vec) = result {
                if !vec.is_empty() {
                    // Connect variable declaration node to its type ascription.
                    connect_type_id(engines, type_ascription.type_id(), graph, entry_node)?;
                }
            }

            // Insert variable only after connecting body.expressions
            // This enables:
            //   let ptr = alloc::<u64>(0);
            //   let ptr = realloc::<u64>(ptr, 0, 2);
            // Where previous ptr is used before adding new ptr to variables.
            graph.namespace.insert_variable(
                name.clone(),
                VariableNamespaceEntry {
                    variable_decl_ix: entry_node,
                },
            );
            result
        }
        ty::TyDecl::ConstantDecl(ty::ConstantDecl { decl_id, .. }) => {
            let const_decl = decl_engine.get_constant(decl_id);
            let ty::TyConstantDecl {
                call_path, value, ..
            } = &*const_decl;
            graph
                .namespace
                .insert_global_constant(call_path.suffix.clone(), entry_node);
            if let Some(value) = &value {
                connect_expression(
                    engines,
                    &value.expression,
                    graph,
                    &[entry_node],
                    exit_node,
                    "constant declaration expression",
                    tree_type,
                    value.span.clone(),
                    options,
                )
            } else {
                Ok(leaves.to_vec())
            }
        }
        ty::TyDecl::ConfigurableDecl(ty::ConfigurableDecl { decl_id, .. }) => {
            let config_decl = decl_engine.get_configurable(decl_id);
            let ty::TyConfigurableDecl {
                call_path,
                value,
                type_ascription,
                ..
            } = &*config_decl;

            graph
                .namespace
                .insert_configurable(call_path.suffix.clone(), entry_node);

            connect_type_id(engines, type_ascription.type_id(), graph, entry_node)?;

            if let Some(value) = &value {
                connect_expression(
                    engines,
                    &value.expression,
                    graph,
                    &[entry_node],
                    exit_node,
                    "configurable declaration expression",
                    tree_type,
                    value.span.clone(),
                    options,
                )
            } else {
                Ok(leaves.to_vec())
            }
        }
        ty::TyDecl::ConstGenericDecl(_) => {
            //This is only called from AstNode
            // where a ConstGenericDecl is unreachable
            unreachable!()
        }
        ty::TyDecl::FunctionDecl(ty::FunctionDecl { decl_id, .. }) => {
            let fn_decl = decl_engine.get_function(decl_id);
            connect_typed_fn_decl(
                engines, &fn_decl, graph, entry_node, span, exit_node, tree_type, options,
            )?;
            Ok(leaves.to_vec())
        }
        ty::TyDecl::TraitDecl(ty::TraitDecl { decl_id, .. }) => {
            let trait_decl = decl_engine.get_trait(decl_id);
            connect_trait_declaration(&trait_decl, graph, entry_node, tree_type);
            Ok(leaves.to_vec())
        }
        ty::TyDecl::AbiDecl(ty::AbiDecl { decl_id, .. }) => {
            let abi_decl = decl_engine.get_abi(decl_id);
            connect_abi_declaration(engines, &abi_decl, graph, entry_node, tree_type)?;
            Ok(leaves.to_vec())
        }
        ty::TyDecl::StructDecl(ty::StructDecl { decl_id, .. }) => {
            let struct_decl = decl_engine.get_struct(decl_id);
            connect_struct_declaration(&struct_decl, *decl_id, graph, entry_node, tree_type);
            Ok(leaves.to_vec())
        }
        ty::TyDecl::EnumDecl(ty::EnumDecl { decl_id, .. }) => {
            let enum_decl = decl_engine.get_enum(decl_id);
            connect_enum_declaration(&enum_decl, *decl_id, graph, entry_node);
            Ok(leaves.to_vec())
        }
        ty::TyDecl::EnumVariantDecl(ty::EnumVariantDecl { enum_ref, .. }) => {
            let enum_decl = decl_engine.get_enum(enum_ref.id());
            connect_enum_declaration(&enum_decl, *enum_ref.id(), graph, entry_node);
            Ok(leaves.to_vec())
        }
        ty::TyDecl::ImplSelfOrTrait(ty::ImplSelfOrTrait { decl_id, .. }) => {
            let impl_trait_decl = decl_engine.get_impl_self_or_trait(decl_id);
            let ty::TyImplSelfOrTrait {
                trait_name,
                items,
                trait_decl_ref,
                implementing_for,
                ..
            } = &*impl_trait_decl;

            connect_impl_trait(
                engines,
                trait_name,
                graph,
                items,
                entry_node,
                tree_type,
                trait_decl_ref,
                implementing_for,
                options,
            )?;
            Ok(leaves.to_vec())
        }
        ty::TyDecl::StorageDecl(ty::StorageDecl { decl_id, .. }) => {
            let storage = decl_engine.get_storage(decl_id);
            connect_storage_declaration(&storage, graph, entry_node, tree_type);
            Ok(leaves.to_vec())
        }
        ty::TyDecl::TypeAliasDecl(ty::TypeAliasDecl { decl_id, .. }) => {
            let type_alias = decl_engine.get_type_alias(decl_id);
            connect_type_alias_declaration(engines, &type_alias, graph, entry_node)?;
            Ok(leaves.to_vec())
        }
        ty::TyDecl::TraitTypeDecl(ty::TraitTypeDecl { .. }) => Ok(leaves.to_vec()),
        ty::TyDecl::ErrorRecovery(..) | ty::TyDecl::GenericTypeForFunctionScope(_) => {
            Ok(leaves.to_vec())
        }
    }
}

/// Connect each individual struct field, and when that field is accessed in a subfield expression,
/// connect that field.
fn connect_struct_declaration<'eng: 'cfg, 'cfg>(
    struct_decl: &ty::TyStructDecl,
    struct_decl_id: DeclId<ty::TyStructDecl>,
    graph: &mut ControlFlowGraph<'cfg>,
    entry_node: NodeIndex,
    tree_type: &TreeType,
) {
    let ty::TyStructDecl {
        call_path,
        fields,
        visibility,
        ..
    } = struct_decl;
    let field_nodes = fields
        .iter()
        .map(|field| {
            (
                field.name.clone(),
                graph.add_node(ControlFlowGraphNode::StructField {
                    struct_decl_id,
                    struct_field_name: field.name.clone(),
                    attributes: field.attributes.clone(),
                }),
            )
        })
        .collect::<Vec<_>>();
    // If this is a library or smart contract, and if this is public, then we want to connect the
    // declaration node itself to the individual fields.
    //
    // this is important because if the struct is public, you want to be able to signal that all
    // fields are accessible by just adding an edge to the struct declaration node
    if matches!(tree_type, TreeType::Contract | TreeType::Library)
        && *visibility == Visibility::Public
    {
        for (_name, node) in &field_nodes {
            graph.add_edge(entry_node, *node, "".into());
        }
    }

    // Now, populate the struct namespace with the location of this struct as well as the indexes
    // of the field names
    graph.namespace.insert_struct(
        call_path.suffix.as_str().to_string(),
        entry_node,
        field_nodes,
    );
}

/// Implementations of traits are top-level things that are not conditional, so
/// we insert an edge from the function's starting point to the declaration to show
/// that the declaration was indeed at some point implemented.
/// Additionally, we insert the trait's methods into the method namespace in order to
/// track which exact methods are dead code.
#[allow(clippy::too_many_arguments)]
fn connect_impl_trait<'eng: 'cfg, 'cfg>(
    engines: &'eng Engines,
    trait_name: &CallPath,
    graph: &mut ControlFlowGraph<'cfg>,
    items: &[TyImplItem],
    entry_node: NodeIndex,
    tree_type: &TreeType,
    trait_decl_ref: &Option<DeclRef<InterfaceDeclId>>,
    implementing_for: &GenericArgument,
    options: NodeConnectionOptions,
) -> Result<(), CompileError> {
    let decl_engine = engines.de();
    // If trait_decl_ref is None then the impl trait is an impl self.
    // Impl self does not have any trait to point to.
    if trait_decl_ref.is_some() {
        let trait_decl_node = graph.namespace.find_trait(trait_name).cloned();
        match trait_decl_node {
            None => {
                let node_ix = graph.add_node("External trait".into());
                graph.add_edge(entry_node, node_ix, "".into());
            }
            Some(trait_decl_node) => {
                graph.add_edge_from_entry(entry_node, "".into());
                graph.add_edge(entry_node, trait_decl_node.trait_idx, "".into());
            }
        };
    }

    connect_type_id(engines, implementing_for.type_id(), graph, entry_node)?;

    let trait_entry = graph.namespace.find_trait(trait_name).cloned();
    // Collect the methods that are directly implemented in the trait.
    let mut trait_items_method_names = Vec::new();
    if let Some(trait_decl_ref) = trait_decl_ref {
        if let InterfaceDeclId::Trait(trait_decl_id) = &trait_decl_ref.id() {
            let trait_decl = decl_engine.get_trait(trait_decl_id);
            for trait_item in trait_decl.items.clone() {
                if let ty::TyTraitItem::Fn(func_decl_ref) = trait_item {
                    let functional_decl_id = decl_engine.get_function(&func_decl_ref);
                    trait_items_method_names.push(functional_decl_id.name.as_str().to_string());
                }
            }
        }
    }
    let mut methods_and_indexes = vec![];
    // insert method declarations into the graph
    for item in items {
        match item {
            TyImplItem::Fn(method_decl_ref) => {
                let fn_decl = decl_engine.get_function(method_decl_ref);
                let fn_decl_entry_node = graph.add_node(ControlFlowGraphNode::MethodDeclaration {
                    span: fn_decl.span.clone(),
                    method_name: fn_decl.name.clone(),
                    method_decl_ref: method_decl_ref.clone(),
                    engines,
                });
                let add_edge_to_fn_decl =
                    if trait_items_method_names.contains(&fn_decl.name.as_str().to_string()) {
                        if let Some(trait_entry) = trait_entry.clone() {
                            matches!(
                                trait_entry.module_tree_type,
                                TreeType::Library | TreeType::Contract
                            )
                        } else {
                            // trait_entry not found which means it is an external trait.
                            // As the trait is external we assume it is within a library
                            // thus we can return true directly.
                            true
                        }
                    } else {
                        matches!(tree_type, TreeType::Library | TreeType::Contract)
                    };
                if add_edge_to_fn_decl {
                    graph.add_edge(entry_node, fn_decl_entry_node, "".into());
                }
                // connect the impl declaration node to the functions themselves, as all trait functions are
                // public if the trait is in scope
                connect_typed_fn_decl(
                    engines,
                    &fn_decl,
                    graph,
                    fn_decl_entry_node,
                    fn_decl.span.clone(),
                    None,
                    tree_type,
                    options,
                )?;
                methods_and_indexes.push((fn_decl.name.clone(), fn_decl_entry_node));
            }
            TyImplItem::Constant(_const_decl) => {}
            TyImplItem::Type(_type_decl) => {}
        }
    }
    // we also want to add an edge from the methods back to the trait, so if a method gets called,
    // the trait impl is considered used
    for (_, ix) in methods_and_indexes.iter() {
        graph.add_edge(*ix, entry_node, "".into());
    }

    // Now, insert the methods into the trait method namespace.
    graph
        .namespace
        .insert_trait_methods(trait_name.clone(), methods_and_indexes);

    Ok(())
}

/// The strategy here is to populate the trait namespace with just one singular trait
/// and if it is ever implemented, by virtue of type checking, we know all interface points
/// were met.
/// Upon implementation, we can populate the methods namespace and track dead functions that way.
/// TL;DR: At this point, we _only_ track the wholistic trait declaration and not the functions
/// contained within.
///
/// The trait node itself has already been added (as `entry_node`), so we just need to insert that
/// node index into the namespace for the trait.
fn connect_trait_declaration(
    decl: &ty::TyTraitDecl,
    graph: &mut ControlFlowGraph,
    entry_node: NodeIndex,
    tree_type: &TreeType,
) {
    graph.namespace.add_trait(
        CallPath {
            prefixes: vec![],
            suffix: decl.name.clone(),
            callpath_type: CallPathType::Ambiguous,
        },
        TraitNamespaceEntry {
            trait_idx: entry_node,
            module_tree_type: *tree_type,
        },
    );
}

/// See [connect_trait_declaration] for implementation details.
fn connect_abi_declaration(
    engines: &Engines,
    decl: &ty::TyAbiDecl,
    graph: &mut ControlFlowGraph,
    entry_node: NodeIndex,
    tree_type: &TreeType,
) -> Result<(), CompileError> {
    let type_engine = engines.te();
    let decl_engine = engines.de();

    graph.namespace.add_trait(
        CallPath {
            prefixes: vec![],
            suffix: decl.name.clone(),
            callpath_type: CallPathType::Ambiguous,
        },
        TraitNamespaceEntry {
            trait_idx: entry_node,
            module_tree_type: *tree_type,
        },
    );

    // If a struct type is used as a return type in the interface surface
    // of the contract, then assume that any fields inside the struct can
    // be used outside of the contract.
    for item in decl.interface_surface.iter() {
        match item {
            ty::TyTraitInterfaceItem::TraitFn(fn_decl_ref) => {
                let fn_decl = decl_engine.get_trait_fn(fn_decl_ref);
                if let Some(TypeInfo::Struct(decl_ref)) = get_struct_type_info_from_type_id(
                    type_engine,
                    decl_engine,
                    fn_decl.return_type.type_id(),
                )? {
                    let decl = decl_engine.get_struct(&decl_ref);
                    if let Some(ns) = graph.namespace.get_struct(&decl.call_path.suffix).cloned() {
                        for (_, field_ix) in ns.fields.iter() {
                            graph.add_edge(ns.struct_decl_ix, *field_ix, "".into());
                        }
                    }
                }
            }
            ty::TyTraitInterfaceItem::Constant(_const_decl) => {}
            ty::TyTraitInterfaceItem::Type(_type_decl) => {}
        }
    }

    Ok(())
}

fn get_struct_type_info_from_type_id(
    type_engine: &TypeEngine,
    decl_engine: &DeclEngine,
    type_id: TypeId,
) -> Result<Option<TypeInfo>, TypeError> {
    let type_info = type_engine.to_typeinfo(type_id, &Span::dummy())?;
    match type_info {
        TypeInfo::Enum(decl_ref) => {
            let decl = decl_engine.get_enum(&decl_ref);
            for p in decl.generic_parameters.iter() {
                let p = p
                    .as_type_parameter()
                    .expect("only works with type parameters");
                if let Ok(Some(type_info)) =
                    get_struct_type_info_from_type_id(type_engine, decl_engine, p.type_id)
                {
                    return Ok(Some(type_info));
                }
            }
            for var in decl.variants.iter() {
                if let Ok(Some(type_info)) = get_struct_type_info_from_type_id(
                    type_engine,
                    decl_engine,
                    var.type_argument.type_id(),
                ) {
                    return Ok(Some(type_info));
                }
            }
            Ok(None)
        }
        TypeInfo::Tuple(type_args) => {
            for arg in type_args.iter() {
                if let Ok(Some(type_info)) =
                    get_struct_type_info_from_type_id(type_engine, decl_engine, arg.type_id())
                {
                    return Ok(Some(type_info));
                }
            }
            Ok(None)
        }
        TypeInfo::Custom { type_arguments, .. } => {
            if let Some(type_arguments) = type_arguments {
                for arg in type_arguments.iter() {
                    if let Ok(Some(type_info)) =
                        get_struct_type_info_from_type_id(type_engine, decl_engine, arg.type_id())
                    {
                        return Ok(Some(type_info));
                    }
                }
            }
            Ok(None)
        }
        TypeInfo::Struct { .. } => Ok(Some(type_info)),
        TypeInfo::Array(type_arg, _) => {
            get_struct_type_info_from_type_id(type_engine, decl_engine, type_arg.type_id())
        }
        TypeInfo::Slice(type_arg) => {
            get_struct_type_info_from_type_id(type_engine, decl_engine, type_arg.type_id())
        }
        _ => Ok(None),
    }
}

/// For an enum declaration, we want to make a declaration node for every individual enum
/// variant. When a variant is constructed, we can point an edge at that variant. This way,
/// we can see clearly, and thusly warn, when individual variants are not ever constructed.
fn connect_enum_declaration<'eng: 'cfg, 'cfg>(
    enum_decl: &ty::TyEnumDecl,
    enum_decl_id: DeclId<ty::TyEnumDecl>,
    graph: &mut ControlFlowGraph<'cfg>,
    entry_node: NodeIndex,
) {
    graph
        .namespace
        .insert_enum(enum_decl.call_path.suffix.clone(), entry_node);

    // keep a mapping of each variant
    for variant in enum_decl.variants.iter() {
        let variant_index = graph.add_node(ControlFlowGraphNode::from_enum_variant(
            enum_decl_id,
            variant.name.clone(),
            enum_decl.visibility != Visibility::Private,
        ));

        graph.namespace.insert_enum_variant(
            enum_decl.call_path.suffix.clone(),
            entry_node,
            variant.name.clone(),
            variant_index,
        );
    }
}

/// When connecting a function declaration, we are inserting a new root node into the graph that
/// has no entry points, since it is just a declaration.
/// When something eventually calls it, it gets connected to the declaration.
#[allow(clippy::too_many_arguments)]
fn connect_typed_fn_decl<'eng: 'cfg, 'cfg>(
    engines: &'eng Engines,
    fn_decl: &ty::TyFunctionDecl,
    graph: &mut ControlFlowGraph<'cfg>,
    entry_node: NodeIndex,
    span: Span,
    exit_node: Option<NodeIndex>,
    tree_type: &TreeType,
    options: NodeConnectionOptions,
) -> Result<(), CompileError> {
    let type_engine = engines.te();

    graph.namespace.push_code_block();
    for fn_param in fn_decl.parameters.iter() {
        let fn_param_node = graph.add_node(ControlFlowGraphNode::FunctionParameter {
            param_name: fn_param.name.clone(),
            is_self: engines
                .te()
                .get(fn_param.type_argument.initial_type_id())
                .is_self_type(),
        });
        graph.add_edge(entry_node, fn_param_node, "".into());

        graph.namespace.insert_variable(
            fn_param.name.clone(),
            VariableNamespaceEntry {
                variable_decl_ix: fn_param_node,
            },
        );

        connect_type_id(
            engines,
            fn_param.type_argument.type_id(),
            graph,
            fn_param_node,
        )?;
    }

    let fn_exit_node = graph.add_node(format!("\"{}\" fn exit", fn_decl.name.as_str()).into());
    let (_exit_nodes, _exit_node) = depth_first_insertion_code_block(
        engines,
        &fn_decl.body,
        graph,
        &[entry_node],
        Some(fn_exit_node),
        tree_type,
        NodeConnectionOptions {
            force_struct_fields_connection: options.force_struct_fields_connection,
            parent_node: Some(entry_node),
        },
    )?;
    graph.namespace.pop_code_block();

    if let Some(exit_node) = exit_node {
        graph.add_edge(fn_exit_node, exit_node, "".into());
    }

    // not sure how correct it is to default to Unit here...
    // I think types should all be resolved by now.
    let ty = type_engine
        .to_typeinfo(fn_decl.return_type.type_id(), &span)
        .unwrap_or_else(|_| TypeInfo::Tuple(Vec::new()));

    let namespace_entry = FunctionNamespaceEntry {
        entry_point: entry_node,
        exit_point: fn_exit_node,
        return_type: ty,
    };

    graph.namespace.insert_function(fn_decl, namespace_entry);

    connect_fn_params_struct_enums(engines, fn_decl, graph, entry_node)?;
    Ok(())
}

// Searches for any structs or enums types referenced by the function
// parameters from the passed function declaration and connects their
// corresponding struct/enum declaration to the function entry node, thus
// making sure they are considered used by the DCA pass.
fn connect_fn_params_struct_enums<'eng: 'cfg, 'cfg>(
    engines: &'eng Engines,
    fn_decl: &ty::TyFunctionDecl,
    graph: &mut ControlFlowGraph<'cfg>,
    fn_decl_entry_node: NodeIndex,
) -> Result<(), CompileError> {
    let type_engine = engines.te();
    for fn_param in fn_decl.parameters.iter() {
        let ty = type_engine.to_typeinfo(
            fn_param.type_argument.type_id(),
            &fn_param.type_argument.span(),
        )?;
        match ty {
            TypeInfo::Enum(decl_ref) => {
                let decl = engines.de().get_enum(&decl_ref);
                let ty_index = match graph.namespace.find_enum(&decl.call_path.suffix) {
                    Some(ix) => *ix,
                    None => graph.add_node(
                        format!("External enum  {}", decl.call_path.suffix.as_str()).into(),
                    ),
                };
                graph.add_edge(fn_decl_entry_node, ty_index, "".into());
            }
            TypeInfo::Struct(decl_ref) => {
                let decl = engines.de().get_struct(&decl_ref);
                let ty_index = match graph
                    .namespace
                    .find_struct_decl(decl.call_path.suffix.as_str())
                {
                    Some(ix) => *ix,
                    None => graph.add_node(
                        format!("External struct  {}", decl.call_path.suffix.as_str()).into(),
                    ),
                };
                graph.add_edge(fn_decl_entry_node, ty_index, "".into());
            }
            _ => {}
        }
    }
    Ok(())
}

fn depth_first_insertion_code_block<'eng: 'cfg, 'cfg>(
    engines: &'eng Engines,
    node_content: &ty::TyCodeBlock,
    graph: &mut ControlFlowGraph<'cfg>,
    leaves: &[NodeIndex],
    exit_node: Option<NodeIndex>,
    tree_type: &TreeType,
    options: NodeConnectionOptions,
) -> Result<(Vec<NodeIndex>, Option<NodeIndex>), CompileError> {
    let mut leaves = leaves.to_vec();
    let mut exit_node = exit_node;
    graph.namespace.push_code_block();
    for node in node_content.contents.iter() {
        let (this_node, l_exit_node) =
            connect_node(engines, node, graph, &leaves, exit_node, tree_type, options)?;
        leaves = this_node;
        exit_node = l_exit_node;
    }
    graph.namespace.pop_code_block();
    Ok((leaves, exit_node))
}

fn get_trait_fn_node_index<'a>(
    engines: &Engines,
    function_decl_ref: DeclRefFunction,
    expression_span: Span,
    graph: &'a ControlFlowGraph,
) -> Result<Option<&'a NodeIndex>, CompileError> {
    let decl_engine = engines.de();
    let fn_decl = decl_engine.get_function(&function_decl_ref);
    if let Some(implementing_type) = &fn_decl.implementing_type {
        match implementing_type {
            ty::TyDecl::TraitDecl(ty::TraitDecl { decl_id, .. }) => {
                let trait_decl = decl_engine.get_trait(decl_id);
                Ok(graph
                    .namespace
                    .find_trait_method(&trait_decl.name.clone().into(), &fn_decl.name))
            }
            ty::TyDecl::StructDecl(ty::StructDecl { decl_id, .. }) => {
                let struct_decl = decl_engine.get_struct(decl_id);
                Ok(graph
                    .namespace
                    .find_trait_method(&struct_decl.call_path.suffix.clone().into(), &fn_decl.name))
            }
            ty::TyDecl::ImplSelfOrTrait(ty::ImplSelfOrTrait { decl_id, .. }) => {
                let impl_trait = decl_engine.get_impl_self_or_trait(decl_id);
                Ok(graph
                    .namespace
                    .find_trait_method(&impl_trait.trait_name, &fn_decl.name))
            }
            ty::TyDecl::AbiDecl(ty::AbiDecl { decl_id, .. }) => {
                let abi_decl = decl_engine.get_abi(decl_id);
                Ok(graph
                    .namespace
                    .find_trait_method(&abi_decl.name.clone().into(), &fn_decl.name))
            }
            _ => Err(CompileError::Internal(
                "Could not get node index for trait function",
                expression_span,
            )),
        }
    } else {
        Ok(None)
    }
}

/// connects any inner parts of an expression to the graph
/// note the main expression node has already been inserted
#[allow(clippy::too_many_arguments)]
fn connect_expression<'eng: 'cfg, 'cfg>(
    engines: &'eng Engines,
    expr_variant: &ty::TyExpressionVariant,
    graph: &mut ControlFlowGraph<'cfg>,
    leaves: &[NodeIndex],
    exit_node: Option<NodeIndex>,
    label: &'static str,
    tree_type: &TreeType,
    expression_span: Span,
    mut options: NodeConnectionOptions,
) -> Result<Vec<NodeIndex>, CompileError> {
    use ty::TyExpressionVariant::*;
    let type_engine = engines.te();
    let decl_engine = engines.de();
    match expr_variant {
        FunctionApplication {
            call_path: name,
            arguments,
            fn_ref,
            contract_call_params,
            selector,
            call_path_typeid,
            contract_caller,
            ..
        } => {
            let fn_decl = decl_engine.get_function(fn_ref);
            let mut is_external = false;

            // in the case of monomorphized functions, first check if we already have a node for
            // it in the namespace. if not then we need to check to see if the namespace contains
            // the decl id parents (the original generic non monomorphized decl id).
            let mut exists = false;
            let parents = decl_engine.find_all_parents(engines, &fn_ref.id().clone());
            for parent in parents.iter() {
                if let Ok(parent_decl_id) = DeclId::try_from(parent) {
                    let parent = decl_engine.get_function(&parent_decl_id);
                    exists |= graph.namespace.get_function(&parent).is_some();
                }
            }

            // find the function in the namespace
            let fn_namespace_entry = graph.namespace.get_function(&fn_decl).cloned();

            // connect function entry point to type in function application call path.
            if let (Some(call_path_typeid), Some(fn_namespace_entry)) =
                (call_path_typeid, fn_namespace_entry.clone())
            {
                connect_type_id(
                    engines,
                    *call_path_typeid,
                    graph,
                    fn_namespace_entry.entry_point,
                )?;
            }

            let mut args_diverge = false;
            for (_name, arg) in arguments {
                if type_engine
                    .get(arg.return_type)
                    .is_uninhabited(engines.te(), engines.de())
                {
                    args_diverge = true;
                }
            }

            let mut param_leaves = leaves.to_vec();
            let mut leaves = if args_diverge {
                vec![]
            } else {
                leaves.to_vec()
            };

            // if the parent node exists in this module, then add the monomorphized version
            // to the graph.
            if fn_namespace_entry.is_none() && exists {
                let (l_leaves, _new_exit_node) = connect_node(
                    engines,
                    &ty::TyAstNode {
                        content: ty::TyAstNodeContent::Declaration(fn_ref.clone().into()),
                        span: expression_span.clone(),
                    },
                    graph,
                    &leaves,
                    exit_node,
                    tree_type,
                    NodeConnectionOptions::default(),
                )?;

                leaves = l_leaves;
            }

            let (fn_entrypoint, fn_exit_point) = fn_namespace_entry
                .map(
                    |FunctionNamespaceEntry {
                         entry_point,
                         exit_point,
                         ..
                     }| (entry_point, exit_point),
                )
                .unwrap_or_else(|| {
                    let node_idx =
                        graph.add_node(format!("extern fn {}()", name.suffix.as_str()).into());
                    is_external = true;
                    (
                        node_idx,
                        graph.add_node(format!("extern fn {} exit", name.suffix.as_str()).into()),
                    )
                });

            let trait_fn_node_idx =
                get_trait_fn_node_index(engines, fn_ref.clone(), expression_span, graph)?;
            if let Some(trait_fn_node_idx) = trait_fn_node_idx {
                if fn_entrypoint != *trait_fn_node_idx {
                    graph.add_edge(fn_entrypoint, *trait_fn_node_idx, "".into());
                }
            }

            for leaf in leaves {
                graph.add_edge(leaf, fn_entrypoint, label.into());
            }

            // save the existing options value to restore after handling the arguments
            let force_struct_fields_connection = options.force_struct_fields_connection;

            // if the function is external, assume that any struct that is being referenced
            // as an argument "consumes" all of the respective struct fields.
            // this could lead to false negatives but it is the best we can do at the moment
            // with our current DCA analysis architecture. revisit this once we switch
            // to an inter-procedural/module analysis approach.
            options.force_struct_fields_connection |= is_external;

            for param_expr in contract_call_params.values() {
                connect_expression(
                    engines,
                    &param_expr.expression,
                    graph,
                    &[fn_entrypoint],
                    exit_node,
                    "",
                    tree_type,
                    param_expr.span.clone(),
                    options,
                )?;
            }

            if let Some(contract_call_params) = selector {
                let mut current_leaf = vec![fn_entrypoint];
                current_leaf = connect_expression(
                    engines,
                    &contract_call_params.contract_caller.expression,
                    graph,
                    &current_leaf,
                    exit_node,
                    "",
                    tree_type,
                    contract_call_params.contract_caller.span.clone(),
                    options,
                )?;
                // connect final leaf to fn exit
                for leaf in current_leaf {
                    graph.add_edge(leaf, fn_exit_point, "".into());
                }
            }

            let mut current_leaf = vec![fn_entrypoint];

            // Connect contract call to contract caller
            if let Some(contract_caller) = contract_caller {
                let span = contract_caller.span.clone();
                connect_expression(
                    engines,
                    &contract_caller.expression,
                    graph,
                    &current_leaf,
                    exit_node,
                    "arg eval",
                    tree_type,
                    span,
                    options,
                )?;
            }

            // we evaluate every one of the function arguments
            for (_name, arg) in arguments {
                let span = arg.span.clone();
                current_leaf = connect_expression(
                    engines,
                    &arg.expression,
                    graph,
                    &param_leaves,
                    exit_node,
                    "arg eval",
                    tree_type,
                    span,
                    options,
                )?;

                if type_engine
                    .get(arg.return_type)
                    .is_uninhabited(engines.te(), engines.de())
                {
                    param_leaves = vec![];
                }
            }
            options.force_struct_fields_connection = force_struct_fields_connection;

            // connect final leaf to fn exit
            for leaf in current_leaf {
                graph.add_edge(leaf, fn_exit_point, "".into());
            }

            // check for std::revert and connect to the exit node if that's the case.
            // we are guaranteed a full call path here since the type checker calls to_fullpath.
            if let Some(prefix) = fn_decl.call_path.prefixes.first() {
                if prefix.as_str() == STD && fn_decl.call_path.suffix.as_str() == "revert" {
                    if let Some(exit_node) = exit_node {
                        graph.add_edge(fn_exit_point, exit_node, "revert".into());
                        return Ok(vec![]);
                    }
                }
            }
            if args_diverge {
                Ok(vec![])
            } else {
                // the exit points get connected to an exit node for the application
                if !is_external {
                    if let Some(exit_node) = exit_node {
                        graph.add_edge(fn_exit_point, exit_node, "".into());
                        Ok(vec![exit_node])
                    } else {
                        Ok(vec![fn_exit_point])
                    }
                } else {
                    Ok(vec![fn_entrypoint])
                }
            }
        }
        LazyOperator { lhs, rhs, .. } => {
            let lhs_expr = connect_expression(
                engines,
                &lhs.expression,
                graph,
                leaves,
                exit_node,
                "",
                tree_type,
                lhs.span.clone(),
                options,
            )?;
            let rhs_expr = connect_expression(
                engines,
                &rhs.expression,
                graph,
                leaves,
                exit_node,
                "",
                tree_type,
                rhs.span.clone(),
                options,
            )?;
            Ok([lhs_expr, rhs_expr].concat())
        }
        Literal(_) => {
            let node = graph.add_node("Literal value".into());
            for leaf in leaves {
                graph.add_edge(*leaf, node, "".into());
            }
            Ok(vec![node])
        }
        VariableExpression { name, .. } => {
            if let Some(variable_entry) = graph.namespace.get_variable(name) {
                for leaf in leaves {
                    graph.add_edge(*leaf, variable_entry.variable_decl_ix, "".into());
                }
                Ok(vec![variable_entry.variable_decl_ix])
            } else {
                // Variables may refer to global const declarations.
                Ok(graph
                    .namespace
                    .get_global_constant(name)
                    .cloned()
                    .map(|node| {
                        for leaf in leaves {
                            graph.add_edge(*leaf, node, "".into());
                        }
                        vec![node]
                    })
                    .unwrap_or_else(|| leaves.to_vec()))
            }
        }
        ConstantExpression { decl, .. } => {
            let node = if let Some(node) = graph.namespace.get_global_constant(decl.name()) {
                *node
            } else if let Some(node) = graph.namespace.get_constant(decl) {
                *node
            } else {
                return Ok(leaves.to_vec());
            };

            for leaf in leaves {
                graph.add_edge(*leaf, node, "".into());
            }
            Ok(vec![node])
        }
        ConfigurableExpression { decl, .. } => {
            let Some(node) = graph.namespace.get_configurable(decl).cloned() else {
                return Ok(leaves.to_vec());
            };

            for leaf in leaves {
                graph.add_edge(*leaf, node, "".into());
            }

            Ok(vec![node])
        }
        ConstGenericExpression { decl, .. } => {
            let Some(node) = graph.namespace.get_const_generic(decl).cloned() else {
                return Ok(leaves.to_vec());
            };

            for leaf in leaves {
                graph.add_edge(*leaf, node, "".into());
            }

            Ok(vec![node])
        }
        EnumInstantiation {
            enum_ref,
            variant_name,
            contents,
            call_path_decl,
            ..
        } => {
            let enum_decl = decl_engine.get_enum(enum_ref);
            // connect this particular instantiation to its variants declaration
            connect_enum_instantiation(
                engines,
                &enum_decl,
                contents,
                variant_name,
                call_path_decl,
                graph,
                leaves,
                exit_node,
                tree_type,
                options,
            )
        }
        MatchExp { desugared, .. } => connect_expression(
            engines,
            &desugared.expression,
            graph,
            leaves,
            exit_node,
            label,
            tree_type,
            expression_span,
            options,
        ),
        IfExp {
            condition,
            then,
            r#else,
        } => {
            let condition_expr = connect_expression(
                engines,
                &condition.expression,
                graph,
                leaves,
                exit_node,
                "",
                tree_type,
                condition.span.clone(),
                options,
            )?;
            let then_expr = connect_expression(
                engines,
                &then.expression,
                graph,
                &condition_expr,
                exit_node,
                "then branch",
                tree_type,
                then.span.clone(),
                options,
            )?;

            let else_expr = if let Some(else_expr) = r#else {
                connect_expression(
                    engines,
                    &else_expr.expression,
                    graph,
                    &condition_expr,
                    exit_node,
                    "else branch",
                    tree_type,
                    else_expr.clone().span,
                    options,
                )?
            } else {
                condition_expr
            };

            Ok([then_expr, else_expr].concat())
        }
        CodeBlock(a @ ty::TyCodeBlock { .. }) => {
            connect_code_block(engines, a, graph, leaves, exit_node, tree_type, options)
        }
        StructExpression {
            struct_id, fields, ..
        } => {
            let struct_decl = engines.de().get_struct(struct_id);
            let decl = match graph
                .namespace
                .find_struct_decl(struct_decl.name().as_str())
            {
                Some(ix) => *ix,
                None => graph.add_node(format!("External struct  {}", struct_decl.name()).into()),
            };
            let entry = graph.add_node("Struct declaration entry".into());
            let exit = graph.add_node("Struct declaration exit".into());
            // connect current leaves to the beginning of this expr
            for leaf in leaves {
                graph.add_edge(*leaf, entry, label.into());
            }
            // connect the entry to the decl, to denote that the struct has been constructed
            graph.add_edge(entry, decl, "".into());

            // connect the struct fields to the struct if its requested as an option
            if options.force_struct_fields_connection {
                if let Some(ns) = graph.namespace.get_struct(struct_decl.name()).cloned() {
                    for (_, field_ix) in ns.fields.iter() {
                        graph.add_edge(decl, *field_ix, "".into());
                    }
                }
            }

            let mut current_leaf = vec![entry];
            // for every field, connect its expression
            for ty::TyStructExpressionField { value, .. } in fields {
                current_leaf = connect_expression(
                    engines,
                    &value.expression,
                    graph,
                    &current_leaf,
                    exit_node,
                    "struct field instantiation",
                    tree_type,
                    value.clone().span,
                    options,
                )?;
            }

            // connect the final field to the exit
            for leaf in current_leaf {
                graph.add_edge(leaf, exit, "".into());
            }
            Ok(vec![exit])
        }
        StructFieldAccess {
            prefix,
            field_to_access,
            resolved_type_of_parent,
            field_instantiation_span,
            ..
        } => {
            connect_expression(
                engines,
                &prefix.expression,
                graph,
                leaves,
                exit_node,
                label,
                tree_type,
                field_instantiation_span.clone(),
                options,
            )?;

            let resolved_type_of_parent = type_engine
                .to_typeinfo(*resolved_type_of_parent, &field_to_access.span)
                .unwrap_or_else(|_| TypeInfo::Tuple(Vec::new()));

            let resolved_type_of_parent = match resolved_type_of_parent
                .expect_struct(&Handler::default(), engines, field_instantiation_span)
                .ok()
            {
                Some(struct_decl_ref) => decl_engine.get_struct(&struct_decl_ref).call_path.clone(),
                None => {
                    return Err(CompileError::Internal(
                        "Called subfield on a non-struct",
                        field_instantiation_span.clone(),
                    ))
                }
            };

            let field_name = &field_to_access.name;
            // find the struct field index in the namespace
            let field_ix = match graph
                .namespace
                .find_struct_field_idx(resolved_type_of_parent.suffix.as_str(), field_name.as_str())
            {
                Some(ix) => *ix,
                None => graph.add_node("external struct".into()),
            };

            let this_ix = graph.add_node(
                format!("Struct field access: {resolved_type_of_parent}.{field_name}").into(),
            );
            for leaf in leaves {
                graph.add_edge(*leaf, this_ix, "".into());
            }

            // autogenerated code should not increase usage of a struct field
            if !engines
                .se()
                .is_span_in_autogenerated(&expression_span)
                .unwrap_or(false)
            {
                graph.add_edge(this_ix, field_ix, "".into());

                if let Some(struct_node_ix) = graph
                    .namespace
                    .find_struct_decl(resolved_type_of_parent.suffix.as_str())
                {
                    graph.add_edge(this_ix, *struct_node_ix, "".into());
                }
            }

            Ok(vec![this_ix])
        }
        AsmExpression { registers, .. } => {
            let asm_node_entry = graph.add_node("Inline asm entry".into());
            let asm_node_exit = graph.add_node("Inline asm exit".into());
            for leaf in leaves {
                graph.add_edge(*leaf, asm_node_entry, "".into());
            }

            let mut current_leaf = vec![asm_node_entry];
            for ty::TyAsmRegisterDeclaration { initializer, .. } in registers {
                current_leaf = match initializer {
                    Some(initializer) => connect_expression(
                        engines,
                        &initializer.expression,
                        graph,
                        &current_leaf,
                        exit_node,
                        "asm block argument initialization",
                        tree_type,
                        initializer.clone().span,
                        options,
                    )?,
                    None => current_leaf,
                }
            }

            // connect the final field to the exit
            for leaf in current_leaf {
                graph.add_edge(leaf, asm_node_exit, "".into());
            }

            Ok(vec![asm_node_exit])
        }
        Tuple { fields } => {
            let entry = graph.add_node("tuple entry".into());
            let exit = graph.add_node("tuple exit".into());
            // connect current leaves to the beginning of this expr
            for leaf in leaves {
                graph.add_edge(*leaf, entry, label.into());
            }

            let mut current_leaf = vec![entry];
            // for every field, connect its expression
            for value in fields {
                current_leaf = connect_expression(
                    engines,
                    &value.expression,
                    graph,
                    &current_leaf,
                    exit_node,
                    "tuple field instantiation",
                    tree_type,
                    value.clone().span,
                    options,
                )?;
            }

            // connect the final field to the exit
            for leaf in current_leaf {
                graph.add_edge(leaf, exit, "".into());
            }
            Ok(vec![exit])
        }
        AbiCast { address, .. } => connect_expression(
            engines,
            &address.expression,
            graph,
            leaves,
            exit_node,
            "abi cast address",
            tree_type,
            address.span.clone(),
            options,
        ),
        ArrayExplicit {
            elem_type: _,
            contents,
        } => {
            let mut last = leaves.to_vec();

            for elem in contents.iter() {
                last = connect_expression(
                    engines,
                    &elem.expression,
                    graph,
                    last.as_slice(),
                    None,
                    "",
                    tree_type,
                    elem.span.clone(),
                    options,
                )?;

                // If an element diverges, break the connections and return nothing
                if last.is_empty() {
                    break;
                }
            }

            Ok(last)
        }
        ArrayRepeat {
            elem_type: _,
            value,
            length,
        } => {
            let value_idx = connect_expression(
                engines,
                &value.expression,
                graph,
                leaves,
                exit_node,
                "",
                tree_type,
                value.span.clone(),
                options,
            )?;
            let length_idx = connect_expression(
                engines,
                &length.expression,
                graph,
                leaves,
                exit_node,
                "",
                tree_type,
                length.span.clone(),
                options,
            )?;
            Ok([value_idx, length_idx].concat())
        }
        ArrayIndex { prefix, index } => {
            let prefix_idx = connect_expression(
                engines,
                &prefix.expression,
                graph,
                leaves,
                exit_node,
                "",
                tree_type,
                prefix.span.clone(),
                options,
            )?;
            let index_idx = connect_expression(
                engines,
                &index.expression,
                graph,
                leaves,
                exit_node,
                "",
                tree_type,
                index.span.clone(),
                options,
            )?;
            Ok([prefix_idx, index_idx].concat())
        }
        TupleElemAccess { prefix, .. } => {
            let prefix_idx = connect_expression(
                engines,
                &prefix.expression,
                graph,
                leaves,
                exit_node,
                "",
                tree_type,
                prefix.span.clone(),
                options,
            )?;
            Ok(prefix_idx)
        }
        StorageAccess(fields) => {
            let storage_node = graph
                .namespace
                .storage
                .get(&fields.storage_field_name())
                .cloned();
            let this_ix = graph
                .add_node(format!("storage field access: {}", fields.storage_field_name()).into());
            for leaf in leaves {
                storage_node.map(|x| graph.add_edge(*leaf, x, "".into()));
                graph.add_edge(*leaf, this_ix, "".into());
            }
            Ok(vec![this_ix])
        }
        IntrinsicFunction(kind) => {
            let prefix_idx =
                connect_intrinsic_function(engines, kind, graph, leaves, exit_node, tree_type)?;
            Ok(prefix_idx)
        }
        AbiName(abi_name) => {
            if let crate::type_system::AbiName::Known(abi_name) = abi_name {
                // abis are treated as traits here
                let entry = graph.namespace.find_trait(abi_name).cloned();
                if let Some(entry) = entry {
                    for leaf in leaves {
                        graph.add_edge(*leaf, entry.trait_idx, "".into());
                    }
                }
            }
            Ok(leaves.to_vec())
        }
        FunctionParameter => Ok(leaves.to_vec()),
        EnumTag { exp } => connect_expression(
            engines,
            &exp.expression,
            graph,
            leaves,
            exit_node,
            "enum tag exp",
            tree_type,
            exp.span.clone(),
            options,
        ),
        UnsafeDowncast {
            exp,
            call_path_decl,
            variant: _,
        } => {
            // Connects call path decl, useful for aliases.
            connect_call_path_decl(engines, call_path_decl, graph, leaves)?;

            connect_expression(
                engines,
                &exp.expression,
                graph,
                leaves,
                exit_node,
                "unsafe downcast exp",
                tree_type,
                exp.span.clone(),
                options,
            )
        }
        WhileLoop {
            body, condition, ..
        } => {
            // a while loop can loop back to the beginning,
            // or it can terminate.
            // so we connect the _end_ of the while loop _both_ to its beginning and the next node.
            // the loop could also be entirely skipped

            let entry = leaves[0];

            let while_loop_exit = graph.add_node("while loop exit".to_string().into());

            let mut leaves = vec![entry];

            if !matches!(*type_engine.get(condition.return_type), TypeInfo::Never) {
                // it is possible for a whole while loop to be skipped so add edge from
                // beginning of while loop straight to exit
                graph.add_edge(
                    entry,
                    while_loop_exit,
                    "condition is initially false".into(),
                );
            } else {
                // As condition return type is NeverType we should not connect the remaining nodes to entry.
                leaves = vec![];
            }

            // handle the condition of the loop
            connect_expression(
                engines,
                &condition.expression,
                graph,
                &leaves,
                exit_node,
                label,
                tree_type,
                Span::dummy(),
                options,
            )?;

            let (l_leaves, _l_exit_node) = depth_first_insertion_code_block(
                engines, body, graph, &leaves, exit_node, tree_type, options,
            )?;
            // insert edges from end of block back to beginning of it
            for leaf in &l_leaves {
                graph.add_edge(*leaf, entry, "loop repeats".into());
            }

            leaves = l_leaves;
            for leaf in leaves {
                graph.add_edge(leaf, while_loop_exit, "".into());
            }
            Ok(vec![while_loop_exit])
        }
        ForLoop { desugared, .. } => connect_expression(
            engines,
            &desugared.expression,
            graph,
            leaves,
            exit_node,
            label,
            tree_type,
            expression_span,
            options,
        ),
        Break => {
            let break_node = graph.add_node("break".to_string().into());
            for leaf in leaves {
                graph.add_edge(*leaf, break_node, "".into());
            }
            Ok(vec![])
        }
        Continue => {
            let continue_node = graph.add_node("continue".to_string().into());
            for leaf in leaves {
                graph.add_edge(*leaf, continue_node, "".into());
            }
            Ok(vec![])
        }
        Reassignment(typed_reassignment) => {
            match &typed_reassignment.lhs {
                ty::TyReassignmentTarget::ElementAccess {
                    base_name, indices, ..
                } => {
                    if let Some(variable_entry) = graph.namespace.get_variable(base_name) {
                        for leaf in leaves {
                            graph.add_edge(
                                *leaf,
                                variable_entry.variable_decl_ix,
                                "variable reassignment LHS".into(),
                            );
                        }
                    };

                    for projection in indices {
                        if let ProjectionKind::ArrayIndex { index, index_span } = projection {
                            connect_expression(
                                engines,
                                &index.expression,
                                graph,
                                leaves,
                                exit_node,
                                "variable reassignment LHS array index",
                                tree_type,
                                index_span.clone(),
                                options,
                            )?;
                        }
                    }
                }
                ty::TyReassignmentTarget::DerefAccess { exp, indices } => {
                    connect_expression(
                        engines,
                        &exp.expression,
                        graph,
                        leaves,
                        exit_node,
                        "variable reassignment LHS dereferencing",
                        tree_type,
                        exp.span.clone(),
                        options,
                    )?;

                    for projection in indices {
                        if let ProjectionKind::ArrayIndex { index, index_span } = projection {
                            connect_expression(
                                engines,
                                &index.expression,
                                graph,
                                leaves,
                                exit_node,
                                "variable reassignment LHS array index",
                                tree_type,
                                index_span.clone(),
                                options,
                            )?;
                        }
                    }
                }
            };

            connect_expression(
                engines,
                &typed_reassignment.rhs.expression,
                graph,
                leaves,
                exit_node,
                "variable reassignment RHS",
                tree_type,
                typed_reassignment.rhs.span.clone(),
                options,
            )
        }
        ImplicitReturn(exp) | Return(exp) | Panic(exp) => {
            let return_type = match expr_variant {
                ImplicitReturn(_) => "implicit return",
                Return(_) => "return",
                Panic(_) => "panic",
                _ => unreachable!(
                    "the `expr_variant` is checked to be `ImplicitReturn`, `Return`, or `Panic`"
                ),
            };

            let this_index = graph.add_node(format!("{return_type} entry").into());
            for leaf in leaves {
                graph.add_edge(*leaf, this_index, "".into());
            }
            let return_contents = connect_expression(
                engines,
                &exp.expression,
                graph,
                &[this_index],
                exit_node,
                "",
                tree_type,
                exp.span.clone(),
                options,
            )?;
            if let Return(_) | Panic(_) = expr_variant {
                // TODO: is this right? Shouldn't we connect the return_contents leaves to the exit
                // node?
                for leaf in return_contents {
                    graph.add_edge(this_index, leaf, "".into());
                }
                if let Some(exit_node) = exit_node {
                    graph.add_edge(this_index, exit_node, return_type.into());
                }
                Ok(vec![])
            } else {
                Ok(return_contents)
            }
        }
        Ref(exp) | Deref(exp) => connect_expression(
            engines,
            &exp.expression,
            graph,
            leaves,
            exit_node,
            "",
            tree_type,
            exp.span.clone(),
            options,
        ),
    }
}

fn connect_intrinsic_function<'eng: 'cfg, 'cfg>(
    engines: &'eng Engines,
    ty::TyIntrinsicFunctionKind {
        kind, arguments, ..
    }: &ty::TyIntrinsicFunctionKind,
    graph: &mut ControlFlowGraph<'cfg>,
    leaves: &[NodeIndex],
    exit_node: Option<NodeIndex>,
    tree_type: &TreeType,
) -> Result<Vec<NodeIndex>, CompileError> {
    let node = graph.add_node(format!("Intrinsic {kind}").into());
    for leaf in leaves {
        graph.add_edge(*leaf, node, "".into());
    }
    let mut result = vec![node];
    let _ = arguments.iter().try_fold(&mut result, |accum, exp| {
        let mut res = connect_expression(
            engines,
            &exp.expression,
            graph,
            leaves,
            exit_node,
            "intrinsic",
            tree_type,
            exp.span.clone(),
            NodeConnectionOptions {
                force_struct_fields_connection: true,
                parent_node: Some(node),
            },
        )?;
        accum.append(&mut res);
        Ok::<_, CompileError>(accum)
    })?;
    if let Some(exit_node) = exit_node {
        if kind == &Intrinsic::Revert {
            graph.add_edge(node, exit_node, "revert".into());
            result = vec![];
        }
    }
    Ok(result)
}

fn connect_code_block<'eng: 'cfg, 'cfg>(
    engines: &'eng Engines,
    block: &ty::TyCodeBlock,
    graph: &mut ControlFlowGraph<'cfg>,
    leaves: &[NodeIndex],
    exit_node: Option<NodeIndex>,
    tree_type: &TreeType,
    options: NodeConnectionOptions,
) -> Result<Vec<NodeIndex>, CompileError> {
    let block_entry = graph.add_node("Code block entry".into());
    for leaf in leaves {
        graph.add_edge(*leaf, block_entry, "".into());
    }
    let current_leaf = vec![block_entry];
    let (l_leaves, _l_exit_node) = depth_first_insertion_code_block(
        engines,
        block,
        graph,
        &current_leaf,
        exit_node,
        tree_type,
        options,
    )?;
    if !l_leaves.is_empty() {
        let block_exit = graph.add_node("Code block exit".into());
        for leaf in l_leaves {
            graph.add_edge(leaf, block_exit, "".into());
        }
        Ok(vec![block_exit])
    } else {
        Ok(vec![])
    }
}

#[allow(clippy::too_many_arguments)]
fn connect_enum_instantiation<'eng: 'cfg, 'cfg>(
    engines: &'eng Engines,
    enum_decl: &ty::TyEnumDecl,
    contents: &Option<Box<ty::TyExpression>>,
    variant_name: &Ident,
    call_path_decl: &ty::TyDecl,
    graph: &mut ControlFlowGraph<'cfg>,
    leaves: &[NodeIndex],
    exit_node: Option<NodeIndex>,
    tree_type: &TreeType,
    options: NodeConnectionOptions,
) -> Result<Vec<NodeIndex>, CompileError> {
    let enum_call_path = enum_decl.call_path.clone();
    let (decl_ix, variant_index) = graph
        .namespace
        .find_enum_variant_index(&enum_call_path.suffix, variant_name)
        .unwrap_or_else(|| {
            let node_idx = graph.add_node(
                format!(
                    "extern enum {}::{}",
                    enum_call_path.suffix.as_str(),
                    variant_name.as_str()
                )
                .into(),
            );
            (node_idx, node_idx)
        });

    let mut is_variant_unreachable = false;
    if let Some(instantiator) = contents {
        if engines
            .te()
            .get(instantiator.return_type)
            .is_uninhabited(engines.te(), engines.de())
        {
            is_variant_unreachable = true;
        }
    }

    let leaves = if is_variant_unreachable {
        vec![]
    } else {
        leaves.to_vec()
    };

    // Connects call path decl, useful for aliases.
    connect_call_path_decl(engines, call_path_decl, graph, &leaves)?;

    // insert organizational nodes for instantiation of enum
    let enum_instantiation_entry_idx = graph.add_node("enum instantiation entry".into());
    let enum_instantiation_exit_idx = graph.add_node("enum instantiation exit".into());

    // connect to declaration node itself to show that the declaration is used
    graph.add_edge(enum_instantiation_entry_idx, decl_ix, "".into());
    for leaf in leaves {
        graph.add_edge(leaf, enum_instantiation_entry_idx, "".into());
    }

    // add edge from the entry of the enum instantiation to the body of the instantiation
    if let Some(instantiator) = contents {
        let instantiator_contents = connect_expression(
            engines,
            &instantiator.expression,
            graph,
            &[enum_instantiation_entry_idx],
            exit_node,
            "",
            tree_type,
            enum_decl.span.clone(),
            options,
        )?;

        for leaf in instantiator_contents {
            graph.add_edge(leaf, enum_instantiation_exit_idx, "".into());
        }
    }

    graph.add_edge(decl_ix, variant_index, "".into());
    if !is_variant_unreachable {
        graph.add_edge(variant_index, enum_instantiation_exit_idx, "".into());
        Ok(vec![enum_instantiation_exit_idx])
    } else {
        Ok(vec![])
    }
}

/// Given a [ty::TyAstNode] that we know is not reached in the graph, construct a warning
/// representing its unreached status. For example, we want to say "this function is never called"
/// if the node is a function declaration, but "this trait is never used" if it is a trait
/// declaration.
fn construct_dead_code_warning_from_node(
    decl_engine: &DeclEngine,
    node: &ty::TyAstNode,
) -> Option<CompileWarning> {
    Some(match node {
        // if this is a function, struct, or trait declaration that is never called, then it is dead
        // code.
        ty::TyAstNode {
            content:
                ty::TyAstNodeContent::Declaration(ty::TyDecl::FunctionDecl(ty::FunctionDecl {
                    decl_id,
                })),
            ..
        } => CompileWarning {
            span: decl_engine.get(decl_id).name.span(),
            warning_content: Warning::DeadFunctionDeclaration,
        },
        ty::TyAstNode {
            content:
                ty::TyAstNodeContent::Declaration(ty::TyDecl::StructDecl(ty::StructDecl { decl_id })),
            ..
        } => CompileWarning {
            span: decl_engine.get(decl_id).name().span(),
            warning_content: Warning::DeadStructDeclaration,
        },
        ty::TyAstNode {
            content:
                ty::TyAstNodeContent::Declaration(ty::TyDecl::EnumDecl(ty::EnumDecl { decl_id })),
            ..
        } => CompileWarning {
            span: decl_engine.get(decl_id).name().span(),
            warning_content: Warning::DeadEnumDeclaration,
        },
        ty::TyAstNode {
            content:
                ty::TyAstNodeContent::Declaration(ty::TyDecl::TraitDecl(ty::TraitDecl { decl_id })),
            ..
        } => CompileWarning {
            span: decl_engine.get(decl_id).name.span(),
            warning_content: Warning::DeadTrait,
        },
        ty::TyAstNode {
            content:
                ty::TyAstNodeContent::Declaration(ty::TyDecl::ConstantDecl(ty::ConstantDecl {
                    decl_id,
                })),
            ..
        } => CompileWarning {
            span: decl_engine.get_constant(decl_id).name().span(),
            warning_content: Warning::DeadDeclaration,
        },
        ty::TyAstNode {
            content:
                ty::TyAstNodeContent::Declaration(ty::TyDecl::ConfigurableDecl(ty::ConfigurableDecl {
                    decl_id,
                })),
            ..
        } => CompileWarning {
            span: decl_engine.get_configurable(decl_id).name().span(),
            warning_content: Warning::DeadDeclaration,
        },
        ty::TyAstNode {
            content: ty::TyAstNodeContent::Declaration(ty::TyDecl::VariableDecl(decl)),
            span,
        } => {
            // In rare cases, variable declaration spans don't have a path, so we need to check for that
            if decl.name.span().source_id().is_some() {
                CompileWarning {
                    span: decl.name.span(),
                    warning_content: Warning::DeadDeclaration,
                }
            } else if span.source_id().is_some() {
                CompileWarning {
                    span: span.clone(),
                    warning_content: Warning::DeadDeclaration,
                }
            } else {
                return None;
            }
        }
        ty::TyAstNode {
            content:
                ty::TyAstNodeContent::Declaration(ty::TyDecl::ImplSelfOrTrait(ty::ImplSelfOrTrait {
                    decl_id,
                    ..
                })),
            span,
        } => {
            let ty::TyImplSelfOrTrait { .. } = &*decl_engine.get_impl_self_or_trait(decl_id);
            CompileWarning {
                span: span.clone(),
                warning_content: Warning::DeadDeclaration,
            }
        }
        ty::TyAstNode {
            content: ty::TyAstNodeContent::Declaration(ty::TyDecl::AbiDecl(_)),
            ..
        } => return None,
        // We handle storage fields individually. There is no need to emit any warnings for the
        // storage declaration itself.
        ty::TyAstNode {
            content: ty::TyAstNodeContent::Declaration(ty::TyDecl::StorageDecl(_)),
            ..
        } => return None,
        // If there is already an error for the declaration, we don't need to emit a dead code warning.
        ty::TyAstNode {
            content: ty::TyAstNodeContent::Declaration(ty::TyDecl::ErrorRecovery(..)),
            ..
        } => return None,
        ty::TyAstNode {
            content: ty::TyAstNodeContent::Declaration(_),
            span,
        } => CompileWarning {
            span: span.clone(),
            warning_content: Warning::DeadDeclaration,
        },
        // Otherwise, this is unreachable.
        ty::TyAstNode {
            span,
            content: ty::TyAstNodeContent::Expression(_) | ty::TyAstNodeContent::SideEffect(_),
        } => CompileWarning {
            span: span.clone(),
            warning_content: Warning::UnreachableCode,
        },
        ty::TyAstNode {
            content: TyAstNodeContent::Error(_, _),
            ..
        } => return None,
    })
}

fn connect_storage_declaration<'eng: 'cfg, 'cfg>(
    decl: &ty::TyStorageDecl,
    graph: &mut ControlFlowGraph<'cfg>,
    _entry_node: NodeIndex,
    _tree_type: &TreeType,
) {
    let ty::TyStorageDecl { fields, .. } = decl;
    let field_nodes = fields
        .iter()
        .map(|field| (field.clone(), graph.add_node(field.into())))
        .collect::<Vec<_>>();

    graph.namespace.insert_storage(field_nodes);
}

fn connect_type_alias_declaration<'eng: 'cfg, 'cfg>(
    engines: &'eng Engines,
    decl: &ty::TyTypeAliasDecl,
    graph: &mut ControlFlowGraph<'cfg>,
    entry_node: NodeIndex,
) -> Result<(), CompileError> {
    graph
        .namespace
        .insert_alias(decl.name().clone(), entry_node);

    let ty::TyTypeAliasDecl { ty, .. } = decl;
    connect_type_id(engines, ty.type_id(), graph, entry_node)?;

    Ok(())
}

fn connect_type_id<'eng: 'cfg, 'cfg>(
    engines: &'eng Engines,
    type_id: TypeId,
    graph: &mut ControlFlowGraph<'cfg>,
    entry_node: NodeIndex,
) -> Result<(), CompileError> {
    let decl_engine = engines.de();
    let type_engine = engines.te();

    match &*type_engine.get(type_id) {
        TypeInfo::Enum(decl_ref) => {
            let decl = decl_engine.get_enum(decl_ref);
            let enum_idx = graph.namespace.find_enum(decl.name());
            if let Some(enum_idx) = enum_idx.cloned() {
                graph.add_edge(entry_node, enum_idx, "".into());
            }
            for p in &decl.generic_parameters {
                match p {
                    crate::TypeParameter::Type(p) => {
                        connect_type_id(engines, p.type_id, graph, entry_node)?;
                    }
                    crate::TypeParameter::Const(_) => {}
                }
            }
        }
        TypeInfo::Struct(decl_ref) => {
            let decl = decl_engine.get_struct(decl_ref);
            let struct_idx = graph.namespace.find_struct_decl(decl.name().as_str());
            if let Some(struct_idx) = struct_idx.cloned() {
                graph.add_edge(entry_node, struct_idx, "".into());
            }
            for p in decl
                .generic_parameters
                .iter()
                .filter_map(|x| x.as_type_parameter())
            {
                connect_type_id(engines, p.type_id, graph, entry_node)?;
            }
        }
        TypeInfo::Alias { name, .. } => {
            let alias_idx = graph.namespace.get_alias(name);
            if let Some(alias_idx) = alias_idx.cloned() {
                graph.add_edge(entry_node, alias_idx, "".into());
            }
        }
        _ => {}
    }

    Ok(())
}

fn connect_call_path_decl<'eng: 'cfg, 'cfg>(
    engines: &'eng Engines,
    call_path_decl: &ty::TyDecl,
    graph: &mut ControlFlowGraph<'cfg>,
    leaves: &[NodeIndex],
) -> Result<(), CompileError> {
    let decl_engine = engines.de();
    if let ty::TyDecl::TypeAliasDecl(ty::TypeAliasDecl { decl_id, .. }) = call_path_decl {
        let decl = decl_engine.get_type_alias(decl_id);
        let alias_idx = graph
            .namespace
            .get_alias(decl.name())
            .cloned()
            .unwrap_or_else(|| graph.add_node(format!("extern alias {}", decl.name()).into()));
        for leaf in leaves {
            graph.add_edge(*leaf, alias_idx, "".into());
        }
    }
    Ok(())
}

/// Checks `attributes` for any `#[allow(dead_code)]` usage, if so returns true
/// otherwise returns false.
fn allow_dead_code(attributes: Attributes) -> bool {
    attributes.has_allow_dead_code()
}

/// Returns true when the given `node` contains the attribute `#[allow(dead_code)]`
fn allow_dead_code_ast_node(decl_engine: &DeclEngine, node: &ty::TyAstNode) -> bool {
    match &node.content {
        ty::TyAstNodeContent::Declaration(decl) => match &decl {
            ty::TyDecl::VariableDecl(_) => false,
            ty::TyDecl::ConstantDecl(ty::ConstantDecl { decl_id, .. }) => {
                allow_dead_code(decl_engine.get_constant(decl_id).attributes.clone())
            }
            ty::TyDecl::ConfigurableDecl(ty::ConfigurableDecl { decl_id, .. }) => {
                allow_dead_code(decl_engine.get_configurable(decl_id).attributes.clone())
            }
            ty::TyDecl::ConstGenericDecl(_) => {
                // only called from AstNode from where
                // ConstGenericDecl is unreachable
                unreachable!()
            }
            ty::TyDecl::TraitTypeDecl(ty::TraitTypeDecl { decl_id, .. }) => {
                allow_dead_code(decl_engine.get_type(decl_id).attributes.clone())
            }
            ty::TyDecl::FunctionDecl(ty::FunctionDecl { decl_id, .. }) => {
                allow_dead_code(decl_engine.get_function(decl_id).attributes.clone())
            }
            ty::TyDecl::TraitDecl(ty::TraitDecl { decl_id, .. }) => {
                allow_dead_code(decl_engine.get_trait(decl_id).attributes.clone())
            }
            ty::TyDecl::StructDecl(ty::StructDecl { decl_id, .. }) => {
                allow_dead_code(decl_engine.get_struct(decl_id).attributes.clone())
            }
            ty::TyDecl::EnumDecl(ty::EnumDecl { decl_id, .. }) => {
                allow_dead_code(decl_engine.get_enum(decl_id).attributes.clone())
            }
            ty::TyDecl::EnumVariantDecl(ty::EnumVariantDecl {
                enum_ref,
                variant_name,
                ..
            }) => {
                let enum_decl = decl_engine.get_enum(enum_ref.id());
                enum_decl
                    .variants
                    .iter()
                    .find(|v| v.name == *variant_name)
                    .map(|enum_variant| allow_dead_code(enum_variant.attributes.clone()))
                    .unwrap_or(false)
            }
            ty::TyDecl::TypeAliasDecl(ty::TypeAliasDecl { decl_id, .. }) => {
                allow_dead_code(decl_engine.get_type_alias(decl_id).attributes.clone())
            }
            ty::TyDecl::ImplSelfOrTrait { .. } => false,
            ty::TyDecl::AbiDecl { .. } => false,
            ty::TyDecl::GenericTypeForFunctionScope { .. } => false,
            ty::TyDecl::ErrorRecovery(..) => false,
            ty::TyDecl::StorageDecl { .. } => false,
        },
        ty::TyAstNodeContent::Expression(_) => false,
        ty::TyAstNodeContent::SideEffect(_) => false,
        ty::TyAstNodeContent::Error(_, _) => false,
    }
}

/// Returns true when the given `node` or its parent contains the attribute `#[allow(dead_code)]`
fn allow_dead_code_node(
    decl_engine: &DeclEngine,
    graph: &Graph,
    node: &ControlFlowGraphNode,
) -> bool {
    match node {
        ControlFlowGraphNode::ProgramNode { node, parent_node } => {
            if let Some(parent_node) = parent_node {
                let parent_node = &graph[*parent_node];
                if allow_dead_code_node(decl_engine, graph, parent_node) {
                    return true;
                }
            }
            allow_dead_code_ast_node(decl_engine, node)
        }
        ControlFlowGraphNode::EnumVariant { enum_decl_id, .. } => {
            allow_dead_code(decl_engine.get_enum(enum_decl_id).attributes.clone())
        }
        ControlFlowGraphNode::MethodDeclaration {
            method_decl_ref, ..
        } => allow_dead_code(decl_engine.get_function(method_decl_ref).attributes.clone()),
        ControlFlowGraphNode::StructField {
            struct_decl_id,
            attributes,
            ..
        } => {
            if allow_dead_code(attributes.clone()) {
                true
            } else {
                allow_dead_code(decl_engine.get_struct(struct_decl_id).attributes.clone())
            }
        }
        ControlFlowGraphNode::StorageField { .. } => false,
        ControlFlowGraphNode::OrganizationalDominator(..) => false,
        ControlFlowGraphNode::FunctionParameter { .. } => false,
    }
}
