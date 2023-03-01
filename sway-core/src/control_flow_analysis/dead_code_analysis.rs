use super::*;
use crate::{
    decl_engine::*,
    language::{
        parsed::TreeType,
        ty::{self, TyImplItem},
        CallPath, Visibility,
    },
    transform::{self, AttributesMap},
    type_system::TypeInfo,
    Engines, TypeEngine, TypeId,
};
use petgraph::{prelude::NodeIndex, visit::Dfs};
use std::collections::BTreeSet;
use sway_error::warning::{CompileWarning, Warning};
use sway_error::{error::CompileError, type_error::TypeError};
use sway_types::{constants::ALLOW_DEAD_CODE_NAME, span::Span, Ident, Spanned};

impl<'cfg> ControlFlowGraph<'cfg> {
    pub(crate) fn find_dead_code(&self, decl_engine: &DeclEngine) -> Vec<CompileWarning> {
        // Dead code is code that has no path from the entry point.
        // Collect all connected nodes by traversing from the entries.
        // The dead nodes are those we did not collect.
        let mut connected = BTreeSet::new();
        let mut dfs = Dfs::empty(&self.graph);
        for &entry in &self.entry_points {
            dfs.move_to(entry);
            while let Some(node) = dfs.next(&self.graph) {
                connected.insert(node);
            }
        }
        let dead_nodes: Vec<_> = self
            .graph
            .node_indices()
            .filter(|n| !connected.contains(n))
            .collect();

        let dead_function_contains_span = |span: &Span| -> bool {
            dead_nodes.iter().any(|x| {
                if let ControlFlowGraphNode::ProgramNode {
                    node:
                        ty::TyAstNode {
                            span: function_span,
                            content:
                                ty::TyAstNodeContent::Declaration(
                                    ty::TyDeclaration::FunctionDeclaration { .. },
                                ),
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
        engines: Engines<'eng>,
        module_nodes: &[ty::TyAstNode],
        tree_type: &TreeType,
        graph: &mut ControlFlowGraph<'cfg>,
        // the `Result` return is just to handle `Unimplemented` errors
    ) -> Result<(), CompileError> {
        // do a depth first traversal and cover individual inner ast nodes
        let decl_engine = engines.de();
        let mut leaves = vec![];
        let exit_node = Some(graph.add_node(("Program exit".to_string()).into()));
        let mut entry_points = vec![];
        let mut non_entry_points = vec![];
        for ast_entrypoint in module_nodes {
            if ast_entrypoint.is_entry_point(decl_engine, tree_type)? {
                entry_points.push(ast_entrypoint);
            } else {
                non_entry_points.push(ast_entrypoint);
            }
        }
        for ast_entrypoint in non_entry_points.into_iter().chain(entry_points) {
            let (l_leaves, _new_exit_node) = connect_node(
                engines,
                ast_entrypoint,
                graph,
                &leaves,
                exit_node,
                tree_type,
                NodeConnectionOptions::default(),
            )?;

            leaves = l_leaves;
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
                node.is_entry_point(decl_engine, tree_type)?
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
    engines: Engines<'eng>,
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
        ty::TyAstNodeContent::ImplicitReturnExpression(expr) => {
            let this_index = graph.add_node(ControlFlowGraphNode::from_node_with_parent(
                node,
                options.parent_node,
            ));
            for leaf_ix in leaves {
                graph.add_edge(*leaf_ix, this_index, "".into());
            }
            // evaluate the expression

            let return_contents = connect_expression(
                engines,
                &expr.expression,
                graph,
                &[this_index],
                exit_node,
                "",
                tree_type,
                expr.span.clone(),
                options,
            )?;

            // connect return to the exit node
            if let Some(exit_node) = exit_node {
                graph.add_edge(this_index, exit_node, "return".into());
            }
            (return_contents, None)
        }
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
                exit_node,
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
    })
}

#[allow(clippy::too_many_arguments)]
fn connect_declaration<'eng: 'cfg, 'cfg>(
    engines: Engines<'eng>,
    decl: &ty::TyDeclaration,
    graph: &mut ControlFlowGraph<'cfg>,
    entry_node: NodeIndex,
    span: Span,
    exit_node: Option<NodeIndex>,
    tree_type: &TreeType,
    leaves: &[NodeIndex],
    options: NodeConnectionOptions,
) -> Result<Vec<NodeIndex>, CompileError> {
    use ty::TyDeclaration::*;
    let decl_engine = engines.de();
    match decl {
        VariableDeclaration(var_decl) => {
            let ty::TyVariableDeclaration { body, .. } = &**var_decl;
            connect_expression(
                engines,
                &body.expression,
                graph,
                &[entry_node],
                exit_node,
                "variable instantiation",
                tree_type,
                body.clone().span,
                options,
            )
        }
        ConstantDeclaration { decl_id, .. } => {
            let ty::TyConstantDeclaration { name, value, .. } = decl_engine.get_constant(decl_id);
            graph.namespace.insert_constant(name, entry_node);
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
        FunctionDeclaration { decl_id, .. } => {
            let fn_decl = decl_engine.get_function(decl_id);
            connect_typed_fn_decl(
                engines, &fn_decl, graph, entry_node, span, exit_node, tree_type, options,
            )?;
            Ok(leaves.to_vec())
        }
        TraitDeclaration { decl_id, .. } => {
            let trait_decl = decl_engine.get_trait(decl_id);
            connect_trait_declaration(&trait_decl, graph, entry_node);
            Ok(leaves.to_vec())
        }
        AbiDeclaration { decl_id, .. } => {
            let abi_decl = decl_engine.get_abi(decl_id);
            connect_abi_declaration(engines, &abi_decl, graph, entry_node)?;
            Ok(leaves.to_vec())
        }
        StructDeclaration { decl_id, .. } => {
            let struct_decl = decl_engine.get_struct(decl_id);
            connect_struct_declaration(&struct_decl, *decl_id, graph, entry_node, tree_type);
            Ok(leaves.to_vec())
        }
        EnumDeclaration { decl_id, .. } => {
            let enum_decl = decl_engine.get_enum(decl_id);
            connect_enum_declaration(&enum_decl, *decl_id, graph, entry_node);
            Ok(leaves.to_vec())
        }
        ImplTrait { decl_id, .. } => {
            let ty::TyImplTrait {
                trait_name, items, ..
            } = decl_engine.get_impl_trait(decl_id);

            connect_impl_trait(
                engines,
                &trait_name,
                graph,
                &items,
                entry_node,
                tree_type,
                options,
            )?;
            Ok(leaves.to_vec())
        }
        StorageDeclaration { decl_id, .. } => {
            let storage = decl_engine.get_storage(decl_id);
            connect_storage_declaration(&storage, graph, entry_node, tree_type);
            Ok(leaves.to_vec())
        }
        ErrorRecovery(_) | GenericTypeForFunctionScope { .. } => Ok(leaves.to_vec()),
    }
}

/// Connect each individual struct field, and when that field is accessed in a subfield expression,
/// connect that field.
fn connect_struct_declaration<'eng: 'cfg, 'cfg>(
    struct_decl: &ty::TyStructDeclaration,
    struct_decl_id: DeclId<ty::TyStructDeclaration>,
    graph: &mut ControlFlowGraph<'cfg>,
    entry_node: NodeIndex,
    tree_type: &TreeType,
) {
    let ty::TyStructDeclaration {
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
                    span: field.span.clone(),
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
    if matches!(tree_type, TreeType::Contract | TreeType::Library { .. })
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
fn connect_impl_trait<'eng: 'cfg, 'cfg>(
    engines: Engines<'eng>,
    trait_name: &CallPath,
    graph: &mut ControlFlowGraph<'cfg>,
    items: &[TyImplItem],
    entry_node: NodeIndex,
    tree_type: &TreeType,
    options: NodeConnectionOptions,
) -> Result<(), CompileError> {
    let decl_engine = engines.de();
    let trait_decl_node = graph.namespace.find_trait(trait_name).cloned();
    match trait_decl_node {
        None => {
            let node_ix = graph.add_node("External trait".into());
            graph.add_edge(entry_node, node_ix, "".into());
        }
        Some(trait_decl_node) => {
            graph.add_edge_from_entry(entry_node, "".into());
            graph.add_edge(entry_node, trait_decl_node, "".into());
        }
    };
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
                if matches!(tree_type, TreeType::Library { .. } | TreeType::Contract) {
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
    decl: &ty::TyTraitDeclaration,
    graph: &mut ControlFlowGraph,
    entry_node: NodeIndex,
) {
    graph.namespace.add_trait(
        CallPath {
            prefixes: vec![],
            suffix: decl.name.clone(),
            is_absolute: false,
        },
        entry_node,
    );
}

/// See [connect_trait_declaration] for implementation details.
fn connect_abi_declaration(
    engines: Engines<'_>,
    decl: &ty::TyAbiDeclaration,
    graph: &mut ControlFlowGraph,
    entry_node: NodeIndex,
) -> Result<(), CompileError> {
    let type_engine = engines.te();
    let decl_engine = engines.de();

    graph.namespace.add_trait(
        CallPath {
            prefixes: vec![],
            suffix: decl.name.clone(),
            is_absolute: false,
        },
        entry_node,
    );

    // If a struct type is used as a return type in the interface surface
    // of the contract, then assume that any fields inside the struct can
    // be used outside of the contract.
    for item in decl.interface_surface.iter() {
        match item {
            ty::TyTraitInterfaceItem::TraitFn(fn_decl_ref) => {
                let fn_decl = decl_engine.get_trait_fn(fn_decl_ref);
                if let Some(TypeInfo::Struct { call_path, .. }) =
                    get_struct_type_info_from_type_id(type_engine, fn_decl.return_type)?
                {
                    if let Some(ns) = graph.namespace.get_struct(&call_path.suffix).cloned() {
                        for (_, field_ix) in ns.fields.iter() {
                            graph.add_edge(ns.struct_decl_ix, *field_ix, "".into());
                        }
                    }
                }
            }
        }
    }

    Ok(())
}

fn get_struct_type_info_from_type_id(
    type_engine: &TypeEngine,
    type_id: TypeId,
) -> Result<Option<TypeInfo>, TypeError> {
    let type_info = type_engine.to_typeinfo(type_id, &Span::dummy())?;
    match type_info {
        TypeInfo::Enum {
            type_parameters,
            variant_types,
            ..
        } => {
            for param in type_parameters.iter() {
                if let Ok(Some(type_info)) =
                    get_struct_type_info_from_type_id(type_engine, param.type_id)
                {
                    return Ok(Some(type_info));
                }
            }
            for var in variant_types.iter() {
                if let Ok(Some(type_info)) =
                    get_struct_type_info_from_type_id(type_engine, var.type_argument.type_id)
                {
                    return Ok(Some(type_info));
                }
            }
            Ok(None)
        }
        TypeInfo::Tuple(type_args) => {
            for arg in type_args.iter() {
                if let Ok(Some(type_info)) =
                    get_struct_type_info_from_type_id(type_engine, arg.type_id)
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
                        get_struct_type_info_from_type_id(type_engine, arg.type_id)
                    {
                        return Ok(Some(type_info));
                    }
                }
            }
            Ok(None)
        }
        TypeInfo::Struct { .. } => Ok(Some(type_info)),
        TypeInfo::Array(type_arg, _) => {
            get_struct_type_info_from_type_id(type_engine, type_arg.type_id)
        }
        _ => Ok(None),
    }
}

/// For an enum declaration, we want to make a declaration node for every individual enum
/// variant. When a variant is constructed, we can point an edge at that variant. This way,
/// we can see clearly, and thusly warn, when individual variants are not ever constructed.
fn connect_enum_declaration<'eng: 'cfg, 'cfg>(
    enum_decl: &ty::TyEnumDeclaration,
    enum_decl_id: DeclId<ty::TyEnumDeclaration>,
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
    engines: Engines<'eng>,
    fn_decl: &ty::TyFunctionDeclaration,
    graph: &mut ControlFlowGraph<'cfg>,
    entry_node: NodeIndex,
    span: Span,
    exit_node: Option<NodeIndex>,
    tree_type: &TreeType,
    options: NodeConnectionOptions,
) -> Result<(), CompileError> {
    let type_engine = engines.te();
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
    if let Some(exit_node) = exit_node {
        graph.add_edge(fn_exit_node, exit_node, "".into());
    }

    // not sure how correct it is to default to Unit here...
    // I think types should all be resolved by now.
    let ty = type_engine
        .to_typeinfo(fn_decl.return_type.type_id, &span)
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
    engines: Engines<'eng>,
    fn_decl: &ty::TyFunctionDeclaration,
    graph: &mut ControlFlowGraph<'cfg>,
    fn_decl_entry_node: NodeIndex,
) -> Result<(), CompileError> {
    let type_engine = engines.te();
    for fn_param in &fn_decl.parameters {
        let ty = type_engine
            .to_typeinfo(fn_param.type_argument.type_id, &fn_param.type_argument.span)?;
        match ty {
            TypeInfo::Enum { call_path, .. } => {
                let ty_index = match graph.namespace.find_enum(&call_path.suffix) {
                    Some(ix) => *ix,
                    None => graph
                        .add_node(format!("External enum  {}", call_path.suffix.as_str()).into()),
                };
                graph.add_edge(fn_decl_entry_node, ty_index, "".into());
            }
            TypeInfo::Struct { call_path, .. } => {
                let ty_index = match graph.namespace.find_struct_decl(call_path.suffix.as_str()) {
                    Some(ix) => *ix,
                    None => graph
                        .add_node(format!("External struct  {}", call_path.suffix.as_str()).into()),
                };
                graph.add_edge(fn_decl_entry_node, ty_index, "".into());
            }
            _ => {}
        }
    }
    Ok(())
}

fn depth_first_insertion_code_block<'eng: 'cfg, 'cfg>(
    engines: Engines<'eng>,
    node_content: &ty::TyCodeBlock,
    graph: &mut ControlFlowGraph<'cfg>,
    leaves: &[NodeIndex],
    exit_node: Option<NodeIndex>,
    tree_type: &TreeType,
    options: NodeConnectionOptions,
) -> Result<(Vec<NodeIndex>, Option<NodeIndex>), CompileError> {
    let mut leaves = leaves.to_vec();
    let mut exit_node = exit_node;
    for node in node_content.contents.iter() {
        let (this_node, l_exit_node) =
            connect_node(engines, node, graph, &leaves, exit_node, tree_type, options)?;
        leaves = this_node;
        exit_node = l_exit_node;
    }
    Ok((leaves, exit_node))
}

fn get_trait_fn_node_index<'a>(
    engines: Engines<'_>,
    function_decl_ref: DeclRefFunction,
    expression_span: Span,
    graph: &'a ControlFlowGraph,
) -> Result<Option<&'a NodeIndex>, CompileError> {
    let decl_engine = engines.de();
    let fn_decl = decl_engine.get_function(&function_decl_ref);
    if let Some(implementing_type) = fn_decl.implementing_type {
        match implementing_type {
            ty::TyDeclaration::TraitDeclaration { decl_id, .. } => {
                let trait_decl = decl_engine.get_trait(&decl_id);
                Ok(graph
                    .namespace
                    .find_trait_method(&trait_decl.name.into(), &fn_decl.name))
            }
            ty::TyDeclaration::StructDeclaration { decl_id, .. } => {
                let struct_decl = decl_engine.get_struct(&decl_id);
                Ok(graph
                    .namespace
                    .find_trait_method(&struct_decl.call_path.suffix.into(), &fn_decl.name))
            }
            ty::TyDeclaration::ImplTrait { decl_id, .. } => {
                let impl_trait = decl_engine.get_impl_trait(&decl_id);
                Ok(graph
                    .namespace
                    .find_trait_method(&impl_trait.trait_name, &fn_decl.name))
            }
            ty::TyDeclaration::AbiDeclaration { decl_id, .. } => {
                let abi_decl = decl_engine.get_abi(&decl_id);
                Ok(graph
                    .namespace
                    .find_trait_method(&abi_decl.name.into(), &fn_decl.name))
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
    engines: Engines<'eng>,
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
            function_decl_ref,
            ..
        } => {
            let fn_decl = decl_engine.get_function(function_decl_ref);
            let mut is_external = false;

            // in the case of monomorphized functions, first check if we already have a node for
            // it in the namespace. if not then we need to check to see if the namespace contains
            // the decl id parents (the original generic non monomorphized decl id).
            let mut exists = false;
            let parents = decl_engine.find_all_parents(engines, &function_decl_ref.id);
            for parent in parents.iter() {
                if let Ok(parent_decl_id) = DeclId::try_from(parent) {
                    let parent = decl_engine.get_function(&parent_decl_id);
                    exists |= graph.namespace.get_function(&parent).is_some();
                }
            }

            // find the function in the namespace
            let fn_namespace_entry = graph.namespace.get_function(&fn_decl).cloned();

            let mut leaves = leaves.to_vec();

            // if the parent node exists in this module, then add the monomorphized version
            // to the graph.
            if fn_namespace_entry.is_none() && exists {
                let (l_leaves, _new_exit_node) = connect_node(
                    engines,
                    &ty::TyAstNode {
                        content: ty::TyAstNodeContent::Declaration(
                            ty::TyDeclaration::FunctionDeclaration {
                                name: function_decl_ref.name.clone(),
                                decl_id: function_decl_ref.id,
                                decl_span: function_decl_ref.decl_span.clone(),
                            },
                        ),
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

            let trait_fn_node_idx = get_trait_fn_node_index(
                engines,
                function_decl_ref.clone(),
                expression_span,
                graph,
            )?;
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

            // we evaluate every one of the function arguments
            let mut current_leaf = vec![fn_entrypoint];
            for (_name, arg) in arguments {
                current_leaf = connect_expression(
                    engines,
                    &arg.expression,
                    graph,
                    &current_leaf,
                    exit_node,
                    "arg eval",
                    tree_type,
                    arg.clone().span,
                    options,
                )?;
            }
            options.force_struct_fields_connection = force_struct_fields_connection;

            // connect final leaf to fn exit
            for leaf in current_leaf {
                graph.add_edge(leaf, fn_exit_point, "".into());
            }
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
            // Variables may refer to global const declarations.
            Ok(graph
                .namespace
                .get_constant(name)
                .cloned()
                .map(|node| {
                    for leaf in leaves {
                        graph.add_edge(*leaf, node, "".into());
                    }
                    vec![node]
                })
                .unwrap_or_else(|| leaves.to_vec()))
        }
        EnumInstantiation {
            enum_decl,
            variant_name,
            contents,
            ..
        } => {
            // connect this particular instantiation to its variants declaration
            connect_enum_instantiation(
                engines,
                enum_decl,
                contents,
                variant_name,
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
            struct_name,
            fields,
            ..
        } => {
            let decl = match graph.namespace.find_struct_decl(struct_name.as_str()) {
                Some(ix) => *ix,
                None => graph.add_node(format!("External struct  {}", struct_name.as_str()).into()),
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
                if let Some(ns) = graph.namespace.get_struct(struct_name).cloned() {
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

            assert!(matches!(resolved_type_of_parent, TypeInfo::Struct { .. }));
            let resolved_type_of_parent = match resolved_type_of_parent {
                TypeInfo::Struct { call_path, .. } => call_path,
                _ => panic!("Called subfield on a non-struct"),
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
            graph.add_edge(this_ix, field_ix, "".into());
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
        Array { contents } => {
            let nodes = contents
                .iter()
                .map(|elem| {
                    connect_expression(
                        engines,
                        &elem.expression,
                        graph,
                        leaves,
                        exit_node,
                        "",
                        tree_type,
                        elem.span.clone(),
                        options,
                    )
                })
                .collect::<Result<Vec<_>, _>>()?;
            Ok(nodes.concat())
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
                let decl = graph.namespace.find_trait(abi_name).cloned();
                if let Some(decl_node) = decl {
                    for leaf in leaves {
                        graph.add_edge(*leaf, decl_node, "".into());
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
        UnsafeDowncast { exp, .. } => connect_expression(
            engines,
            &exp.expression,
            graph,
            leaves,
            exit_node,
            "unsafe downcast exp",
            tree_type,
            exp.span.clone(),
            options,
        ),
        WhileLoop {
            body, condition, ..
        } => {
            // a while loop can loop back to the beginning,
            // or it can terminate.
            // so we connect the _end_ of the while loop _both_ to its beginning and the next node.
            // the loop could also be entirely skipped

            let entry = leaves[0];

            let while_loop_exit = graph.add_node("while loop exit".to_string().into());

            // it is possible for a whole while loop to be skipped so add edge from
            // beginning of while loop straight to exit
            graph.add_edge(
                entry,
                while_loop_exit,
                "condition is initially false".into(),
            );
            let mut leaves = vec![entry];

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
        Reassignment(typed_reassignment) => connect_expression(
            engines,
            &typed_reassignment.rhs.expression,
            graph,
            leaves,
            exit_node,
            "variable reassignment",
            tree_type,
            typed_reassignment.rhs.clone().span,
            options,
        ),
        StorageReassignment(typed_storage_reassignment) => connect_expression(
            engines,
            &typed_storage_reassignment.rhs.expression,
            graph,
            leaves,
            exit_node,
            "variable reassignment",
            tree_type,
            typed_storage_reassignment.rhs.clone().span,
            options,
        ),
        Return(exp) => {
            let this_index = graph.add_node("return entry".into());
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
            // TODO: is this right? Shouldn't we connect the return_contents leaves to the exit
            // node?
            for leaf in return_contents {
                graph.add_edge(this_index, leaf, "".into());
            }
            if let Some(exit_node) = exit_node {
                graph.add_edge(this_index, exit_node, "return".into());
            }
            Ok(vec![])
        }
    }
}

fn connect_intrinsic_function<'eng: 'cfg, 'cfg>(
    engines: Engines<'eng>,
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
    Ok(result)
}

fn connect_code_block<'eng: 'cfg, 'cfg>(
    engines: Engines<'eng>,
    block: &ty::TyCodeBlock,
    graph: &mut ControlFlowGraph<'cfg>,
    leaves: &[NodeIndex],
    exit_node: Option<NodeIndex>,
    tree_type: &TreeType,
    options: NodeConnectionOptions,
) -> Result<Vec<NodeIndex>, CompileError> {
    let contents = &block.contents;
    let block_entry = graph.add_node("Code block entry".into());
    for leaf in leaves {
        graph.add_edge(*leaf, block_entry, "".into());
    }
    let mut current_leaf = vec![block_entry];
    for node in contents {
        current_leaf = connect_node(
            engines,
            node,
            graph,
            &current_leaf,
            exit_node,
            tree_type,
            options,
        )?
        .0;
    }

    let block_exit = graph.add_node("Code block exit".into());
    for leaf in current_leaf {
        graph.add_edge(leaf, block_exit, "".into());
    }
    Ok(vec![block_exit])
}

#[allow(clippy::too_many_arguments)]
fn connect_enum_instantiation<'eng: 'cfg, 'cfg>(
    engines: Engines<'eng>,
    enum_decl: &ty::TyEnumDeclaration,
    contents: &Option<Box<ty::TyExpression>>,
    variant_name: &Ident,
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

    // insert organizational nodes for instantiation of enum
    let enum_instantiation_entry_idx = graph.add_node("enum instantiation entry".into());
    let enum_instantiation_exit_idx = graph.add_node("enum instantiation exit".into());

    // connect to declaration node itself to show that the declaration is used
    graph.add_edge(enum_instantiation_entry_idx, decl_ix, "".into());
    for leaf in leaves {
        graph.add_edge(*leaf, enum_instantiation_entry_idx, "".into());
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
    graph.add_edge(variant_index, enum_instantiation_exit_idx, "".into());

    Ok(vec![enum_instantiation_exit_idx])
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
                ty::TyAstNodeContent::Declaration(ty::TyDeclaration::FunctionDeclaration {
                    name, ..
                }),
            ..
        } => CompileWarning {
            span: name.span(),
            warning_content: Warning::DeadFunctionDeclaration,
        },
        ty::TyAstNode {
            content:
                ty::TyAstNodeContent::Declaration(ty::TyDeclaration::StructDeclaration { name, .. }),
            ..
        } => CompileWarning {
            span: name.span(),
            warning_content: Warning::DeadStructDeclaration,
        },
        ty::TyAstNode {
            content:
                ty::TyAstNodeContent::Declaration(ty::TyDeclaration::EnumDeclaration { name, .. }),
            ..
        } => CompileWarning {
            span: name.span(),
            warning_content: Warning::DeadEnumDeclaration,
        },
        ty::TyAstNode {
            content:
                ty::TyAstNodeContent::Declaration(ty::TyDeclaration::TraitDeclaration { name, .. }),
            ..
        } => CompileWarning {
            span: name.span(),
            warning_content: Warning::DeadTrait,
        },
        ty::TyAstNode {
            content:
                ty::TyAstNodeContent::Declaration(ty::TyDeclaration::ConstantDeclaration {
                    name, ..
                }),
            ..
        } => CompileWarning {
            span: name.span(),
            warning_content: Warning::DeadDeclaration,
        },
        ty::TyAstNode {
            content: ty::TyAstNodeContent::Declaration(ty::TyDeclaration::VariableDeclaration(decl)),
            span,
        } => {
            // In rare cases, variable declaration spans don't have a path, so we need to check for that
            if decl.name.span().path().is_some() {
                CompileWarning {
                    span: decl.name.span(),
                    warning_content: Warning::DeadDeclaration,
                }
            } else if span.path().is_some() {
                CompileWarning {
                    span: span.clone(),
                    warning_content: Warning::DeadDeclaration,
                }
            } else {
                return None;
            }
        }
        ty::TyAstNode {
            content: ty::TyAstNodeContent::Declaration(ty::TyDeclaration::ImplTrait { decl_id, .. }),
            span,
        } => {
            let ty::TyImplTrait { items: methods, .. } = decl_engine.get_impl_trait(decl_id);
            if methods.is_empty() {
                return None;
            } else {
                CompileWarning {
                    span: span.clone(),
                    warning_content: Warning::DeadDeclaration,
                }
            }
        }
        ty::TyAstNode {
            content: ty::TyAstNodeContent::Declaration(ty::TyDeclaration::AbiDeclaration { .. }),
            ..
        } => return None,
        // We handle storage fields individually. There is no need to emit any warnings for the
        // storage declaration itself.
        ty::TyAstNode {
            content: ty::TyAstNodeContent::Declaration(ty::TyDeclaration::StorageDeclaration { .. }),
            ..
        } => return None,
        // If there is already an error for the declaration, we don't need to emit a dead code warning.
        ty::TyAstNode {
            content: ty::TyAstNodeContent::Declaration(ty::TyDeclaration::ErrorRecovery(..)),
            ..
        } => return None,
        ty::TyAstNode {
            content: ty::TyAstNodeContent::Declaration(..),
            span,
        } => CompileWarning {
            span: span.clone(),
            warning_content: Warning::DeadDeclaration,
        },
        // Otherwise, this is unreachable.
        ty::TyAstNode {
            span,
            content:
                ty::TyAstNodeContent::ImplicitReturnExpression(_)
                | ty::TyAstNodeContent::Expression(_)
                | ty::TyAstNodeContent::SideEffect(_),
        } => CompileWarning {
            span: span.clone(),
            warning_content: Warning::UnreachableCode,
        },
    })
}

fn connect_storage_declaration<'eng: 'cfg, 'cfg>(
    decl: &ty::TyStorageDeclaration,
    graph: &mut ControlFlowGraph<'cfg>,
    _entry_node: NodeIndex,
    _tree_type: &TreeType,
) {
    let ty::TyStorageDeclaration { fields, .. } = decl;
    let field_nodes = fields
        .iter()
        .map(|field| (field.clone(), graph.add_node(field.into())))
        .collect::<Vec<_>>();

    graph.namespace.insert_storage(field_nodes);
}

/// Checks [AttributesMap] for `#[allow(dead_code)]` usage, if so returns true
/// otherwise returns false.
fn allow_dead_code(attributes: AttributesMap) -> bool {
    fn allow_dead_code_helper(attributes: AttributesMap) -> Option<bool> {
        Some(
            attributes
                .get(&transform::AttributeKind::Allow)?
                .last()?
                .args
                .first()?
                .as_str()
                == ALLOW_DEAD_CODE_NAME,
        )
    }

    allow_dead_code_helper(attributes).unwrap_or_default()
}

/// Returns true when the given `node` contains the attribute `#[allow(dead_code)]`
fn allow_dead_code_ast_node(decl_engine: &DeclEngine, node: &ty::TyAstNode) -> bool {
    match &node.content {
        ty::TyAstNodeContent::Declaration(decl) => match &decl {
            ty::TyDeclaration::VariableDeclaration(_) => false,
            ty::TyDeclaration::ConstantDeclaration { decl_id, .. } => {
                allow_dead_code(decl_engine.get_constant(decl_id).attributes)
            }
            ty::TyDeclaration::FunctionDeclaration { decl_id, .. } => {
                allow_dead_code(decl_engine.get_function(decl_id).attributes)
            }
            ty::TyDeclaration::TraitDeclaration { decl_id, .. } => {
                allow_dead_code(decl_engine.get_trait(decl_id).attributes)
            }
            ty::TyDeclaration::StructDeclaration { decl_id, .. } => {
                allow_dead_code(decl_engine.get_struct(decl_id).attributes)
            }
            ty::TyDeclaration::EnumDeclaration { decl_id, .. } => {
                allow_dead_code(decl_engine.get_enum(decl_id).attributes)
            }
            ty::TyDeclaration::ImplTrait { .. } => false,
            ty::TyDeclaration::AbiDeclaration { .. } => false,
            ty::TyDeclaration::GenericTypeForFunctionScope { .. } => false,
            ty::TyDeclaration::ErrorRecovery(_) => false,
            ty::TyDeclaration::StorageDeclaration { .. } => false,
        },
        ty::TyAstNodeContent::Expression(_) => false,
        ty::TyAstNodeContent::ImplicitReturnExpression(_) => false,
        ty::TyAstNodeContent::SideEffect(_) => false,
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
            allow_dead_code(decl_engine.get_enum(enum_decl_id).attributes)
        }
        ControlFlowGraphNode::MethodDeclaration {
            method_decl_ref, ..
        } => allow_dead_code(decl_engine.get_function(method_decl_ref).attributes),
        ControlFlowGraphNode::StructField {
            struct_decl_id,
            attributes,
            ..
        } => {
            if allow_dead_code(attributes.clone()) {
                true
            } else {
                allow_dead_code(decl_engine.get_struct(struct_decl_id).attributes)
            }
        }
        ControlFlowGraphNode::StorageField { .. } => false,
        ControlFlowGraphNode::OrganizationalDominator(..) => false,
    }
}
