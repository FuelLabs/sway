use super::*;

use crate::{
    parse_tree::{CallPath, Visibility},
    semantic_analysis::{
        ast_node::{
            TypedAbiDeclaration, TypedCodeBlock, TypedConstantDeclaration, TypedDeclaration,
            TypedEnumDeclaration, TypedExpression, TypedExpressionVariant,
            TypedFunctionDeclaration, TypedReassignment, TypedReturnStatement,
            TypedStructDeclaration, TypedStructExpressionField, TypedTraitDeclaration,
            TypedVariableDeclaration, TypedWhileLoop,
        },
        TypeCheckedStorageReassignment, TypedAstNode, TypedAstNodeContent, TypedParseTree,
    },
    type_engine::{resolve_type, TypeInfo},
    CompileError, CompileWarning, Ident, TreeType, Warning,
};
use sway_types::span::Span;

use crate::semantic_analysis::TypedStorageDeclaration;

use petgraph::algo::has_path_connecting;
use petgraph::prelude::NodeIndex;

impl ControlFlowGraph {
    pub(crate) fn find_dead_code(&self) -> Vec<CompileWarning> {
        // dead code is code that has no path to the entry point
        let mut dead_nodes = vec![];
        for destination in self.graph.node_indices() {
            let mut is_connected = false;
            for entry in &self.entry_points {
                if has_path_connecting(&self.graph, *entry, destination, None) {
                    is_connected = true;
                    break;
                }
            }
            if !is_connected {
                dead_nodes.push(destination);
            }
        }
        let dead_enum_variant_warnings = dead_nodes
            .iter()
            .filter_map(|x| match &self.graph[*x] {
                ControlFlowGraphNode::EnumVariant {
                    span,
                    variant_name,
                    is_public,
                } if !is_public => Some(CompileWarning {
                    span: span.clone(),
                    warning_content: Warning::DeadEnumVariant {
                        variant_name: variant_name.to_string(),
                    },
                }),
                _ => None,
            })
            .collect::<Vec<_>>();

        let dead_ast_node_warnings = dead_nodes
            .into_iter()
            .filter_map(|x| match &self.graph[x] {
                ControlFlowGraphNode::ProgramNode(node) => {
                    construct_dead_code_warning_from_node(node)
                }
                ControlFlowGraphNode::EnumVariant {
                    span,
                    variant_name,
                    is_public,
                } if !is_public => Some(CompileWarning {
                    span: span.clone(),
                    warning_content: Warning::DeadEnumVariant {
                        variant_name: variant_name.to_string(),
                    },
                }),
                ControlFlowGraphNode::EnumVariant { .. } => None,
                ControlFlowGraphNode::MethodDeclaration { span, .. } => Some(CompileWarning {
                    span: span.clone(),
                    warning_content: Warning::DeadMethod,
                }),
                ControlFlowGraphNode::StructField { span, .. } => Some(CompileWarning {
                    span: span.clone(),
                    warning_content: Warning::StructFieldNeverRead,
                }),
                ControlFlowGraphNode::StorageField { field_name, .. } => Some(CompileWarning {
                    span: field_name.span().clone(),
                    warning_content: Warning::DeadStorageDeclaration,
                }),
                ControlFlowGraphNode::OrganizationalDominator(..) => None,
            })
            .collect::<Vec<_>>();

        let all_warnings = [dead_enum_variant_warnings, dead_ast_node_warnings].concat();
        // filter out any overlapping spans -- if a span is contained within another one,
        // remove it.
        all_warnings
            .clone()
            .into_iter()
            .filter(|CompileWarning { span, .. }| {
                // if any other warnings contain a span which completely covers this one, filter
                // out this one.
                all_warnings.iter().any(
                    |CompileWarning {
                         span: other_span, ..
                     }| {
                        other_span.end() > span.end() && other_span.start() < span.start()
                    },
                )
            })
            .collect()
    }

    pub(crate) fn append_to_dead_code_graph(
        ast: &TypedParseTree,
        tree_type: &TreeType,
        graph: &mut ControlFlowGraph,
        // the `Result` return is just to handle `Unimplemented` errors
    ) -> Result<(), CompileError> {
        // do a depth first traversal and cover individual inner ast nodes
        let mut leaves = vec![];
        let exit_node = Some(graph.add_node(("Program exit".to_string()).into()));
        for ast_entrypoint in ast.all_nodes().iter() {
            let (l_leaves, _new_exit_node) =
                connect_node(ast_entrypoint, graph, &leaves, exit_node, tree_type)?;

            leaves = l_leaves;
        }

        // calculate the entry points based on the tree type
        graph.entry_points = match tree_type {
            TreeType::Predicate | TreeType::Script => {
                // a predicate or script have a main function as the only entry point
                vec![
                    graph
                        .graph
                        .node_indices()
                        .find(|i| match graph.graph[*i] {
                            ControlFlowGraphNode::OrganizationalDominator(_) => false,
                            ControlFlowGraphNode::ProgramNode(TypedAstNode {
                                content:
                                    TypedAstNodeContent::Declaration(
                                        TypedDeclaration::FunctionDeclaration(
                                            TypedFunctionDeclaration { ref name, .. },
                                        ),
                                    ),
                                ..
                            }) => name.as_str() == "main",
                            _ => false,
                        })
                        .unwrap(),
                ]
            }
            TreeType::Contract | TreeType::Library { .. } => graph
                .graph
                .node_indices()
                .filter(|i| match graph.graph[*i] {
                    ControlFlowGraphNode::OrganizationalDominator(_) => false,
                    ControlFlowGraphNode::ProgramNode(TypedAstNode {
                        content:
                            TypedAstNodeContent::Declaration(TypedDeclaration::FunctionDeclaration(
                                TypedFunctionDeclaration {
                                    visibility: Visibility::Public,
                                    ..
                                },
                            )),
                        ..
                    }) => true,
                    ControlFlowGraphNode::ProgramNode(TypedAstNode {
                        content:
                            TypedAstNodeContent::Declaration(TypedDeclaration::TraitDeclaration(
                                TypedTraitDeclaration {
                                    visibility: Visibility::Public,
                                    ..
                                },
                            )),
                        ..
                    }) => true,
                    ControlFlowGraphNode::ProgramNode(TypedAstNode {
                        content:
                            TypedAstNodeContent::Declaration(TypedDeclaration::StructDeclaration(
                                TypedStructDeclaration {
                                    visibility: Visibility::Public,
                                    ..
                                },
                            )),
                        ..
                    }) => true,
                    ControlFlowGraphNode::ProgramNode(TypedAstNode {
                        content:
                            TypedAstNodeContent::Declaration(TypedDeclaration::ImplTrait { .. }),
                        ..
                    }) => true,
                    _ => false,
                })
                .collect(),
        };
        Ok(())
    }
}
fn connect_node(
    node: &TypedAstNode,
    graph: &mut ControlFlowGraph,
    leaves: &[NodeIndex],
    exit_node: Option<NodeIndex>,
    tree_type: &TreeType,
) -> Result<(Vec<NodeIndex>, Option<NodeIndex>), CompileError> {
    //    let mut graph = graph.clone();
    let span = node.span.clone();
    Ok(match &node.content {
        TypedAstNodeContent::ReturnStatement(TypedReturnStatement { expr }) => {
            let this_index = graph.add_node(node.into());
            for leaf_ix in leaves {
                graph.add_edge(*leaf_ix, this_index, "".into());
            }
            // evaluate the expression

            let return_contents = connect_expression(
                &expr.expression,
                graph,
                &[this_index],
                exit_node,
                "",
                tree_type,
                expr.span.clone(),
            )?;
            for leaf in return_contents {
                graph.add_edge(this_index, leaf, "".into());
            }
            // connect return to the exit node
            if let Some(exit_node) = exit_node {
                graph.add_edge(this_index, exit_node, "return".into());
                (vec![], None)
            } else {
                (vec![], None)
            }
        }
        TypedAstNodeContent::ImplicitReturnExpression(expr) => {
            let this_index = graph.add_node(node.into());
            for leaf_ix in leaves {
                graph.add_edge(*leaf_ix, this_index, "".into());
            }
            // evaluate the expression

            let return_contents = connect_expression(
                &expr.expression,
                graph,
                &[this_index],
                exit_node,
                "",
                tree_type,
                expr.span.clone(),
            )?;
            for leaf in return_contents.clone() {
                graph.add_edge(this_index, leaf, "".into());
            }
            // connect return to the exit node
            if let Some(exit_node) = exit_node {
                graph.add_edge(this_index, exit_node, "return".into());
            }
            (return_contents, None)
        }
        TypedAstNodeContent::WhileLoop(TypedWhileLoop { body, .. }) => {
            // a while loop can loop back to the beginning,
            // or it can terminate.
            // so we connect the _end_ of the while loop _both_ to its beginning and the next node.
            // the loop could also be entirely skipped

            let entry = graph.add_node(node.into());
            let while_loop_exit = graph.add_node("while loop exit".to_string().into());
            for leaf in leaves {
                graph.add_edge(*leaf, entry, "".into());
            }
            // it is possible for a whole while loop to be skipped so add edge from
            // beginning of while loop straight to exit
            graph.add_edge(
                entry,
                while_loop_exit,
                "condition is initially false".into(),
            );
            let mut leaves = vec![entry];
            let (l_leaves, _l_exit_node) =
                depth_first_insertion_code_block(body, graph, &leaves, exit_node, tree_type)?;
            // insert edges from end of block back to beginning of it
            for leaf in &l_leaves {
                graph.add_edge(*leaf, entry, "loop repeats".into());
            }

            leaves = l_leaves;
            for leaf in leaves {
                graph.add_edge(leaf, while_loop_exit, "".into());
            }
            (vec![while_loop_exit], exit_node)
        }
        TypedAstNodeContent::Expression(TypedExpression {
            expression: expr_variant,
            span,
            ..
        }) => {
            let entry = graph.add_node(node.into());
            // insert organizational dominator node
            // connected to all current leaves
            for leaf in leaves {
                graph.add_edge(*leaf, entry, "".into());
            }

            (
                connect_expression(
                    expr_variant,
                    graph,
                    &[entry],
                    exit_node,
                    "",
                    tree_type,
                    span.clone(),
                )?,
                exit_node,
            )
        }
        TypedAstNodeContent::SideEffect => (leaves.to_vec(), exit_node),
        TypedAstNodeContent::Declaration(decl) => {
            // all leaves connect to this node, then this node is the singular leaf
            let decl_node = graph.add_node(node.into());
            for leaf in leaves {
                graph.add_edge(*leaf, decl_node, "".into());
            }
            (
                connect_declaration(decl, graph, decl_node, span, exit_node, tree_type, leaves)?,
                exit_node,
            )
        }
    })
}

fn connect_declaration(
    decl: &TypedDeclaration,
    graph: &mut ControlFlowGraph,
    entry_node: NodeIndex,
    span: Span,
    exit_node: Option<NodeIndex>,
    tree_type: &TreeType,
    leaves: &[NodeIndex],
) -> Result<Vec<NodeIndex>, CompileError> {
    use TypedDeclaration::*;
    match decl {
        VariableDeclaration(TypedVariableDeclaration { body, .. }) => connect_expression(
            &body.expression,
            graph,
            &[entry_node],
            exit_node,
            "variable instantiation",
            tree_type,
            body.clone().span,
        ),
        ConstantDeclaration(TypedConstantDeclaration { name, .. }) => {
            graph.namespace.insert_constant(name.clone(), entry_node);
            Ok(leaves.to_vec())
        }
        FunctionDeclaration(fn_decl) => {
            connect_typed_fn_decl(fn_decl, graph, entry_node, span, exit_node, tree_type)?;
            Ok(leaves.to_vec())
        }
        TraitDeclaration(trait_decl) => {
            connect_trait_declaration(trait_decl, graph, entry_node);
            Ok(leaves.to_vec())
        }
        AbiDeclaration(abi_decl) => {
            connect_abi_declaration(abi_decl, graph, entry_node);
            Ok(leaves.to_vec())
        }
        StructDeclaration(struct_decl) => {
            connect_struct_declaration(struct_decl, graph, entry_node, tree_type);
            Ok(leaves.to_vec())
        }
        EnumDeclaration(enum_decl) => {
            connect_enum_declaration(enum_decl, graph, entry_node);
            Ok(leaves.to_vec())
        }
        StorageReassignment(TypeCheckedStorageReassignment { rhs, .. }) => connect_expression(
            &rhs.expression,
            graph,
            &[entry_node],
            exit_node,
            "variable reassignment",
            tree_type,
            rhs.span.clone(),
        ),
        Reassignment(TypedReassignment { rhs, .. }) => connect_expression(
            &rhs.expression,
            graph,
            &[entry_node],
            exit_node,
            "variable reassignment",
            tree_type,
            rhs.clone().span,
        ),
        ImplTrait {
            trait_name,
            methods,
            ..
        } => {
            connect_impl_trait(trait_name, graph, methods, entry_node, tree_type)?;
            Ok(leaves.to_vec())
        }
        StorageDeclaration(storage) => {
            connect_storage_declaration(storage, graph, entry_node, tree_type);
            Ok(leaves.to_vec())
        }
        ErrorRecovery | GenericTypeForFunctionScope { .. } => Ok(leaves.to_vec()),
    }
}

/// Connect each individual struct field, and when that field is accessed in a subfield expression,
/// connect that field.
fn connect_struct_declaration(
    struct_decl: &TypedStructDeclaration,
    graph: &mut ControlFlowGraph,
    entry_node: NodeIndex,
    tree_type: &TreeType,
) {
    let TypedStructDeclaration {
        name,
        fields,
        visibility,
        ..
    } = struct_decl;
    let field_nodes = fields
        .iter()
        .map(|field| (field.name.clone(), graph.add_node(field.into())))
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
    graph
        .namespace
        .insert_struct(name.as_str().to_string(), entry_node, field_nodes);
}

/// Implementations of traits are top-level things that are not conditional, so
/// we insert an edge from the function's starting point to the declaration to show
/// that the declaration was indeed at some point implemented.
/// Additionally, we insert the trait's methods into the method namespace in order to
/// track which exact methods are dead code.
fn connect_impl_trait(
    trait_name: &CallPath,
    graph: &mut ControlFlowGraph,
    methods: &[TypedFunctionDeclaration],
    entry_node: NodeIndex,
    tree_type: &TreeType,
) -> Result<(), CompileError> {
    let graph_c = graph.clone();
    let trait_decl_node = graph_c.namespace.find_trait(trait_name);
    match trait_decl_node {
        None => {
            let node_ix = graph.add_node("External trait".into());
            graph.add_edge(entry_node, node_ix, "".into());
        }
        Some(trait_decl_node) => {
            graph.add_edge_from_entry(entry_node, "".into());
            graph.add_edge(entry_node, *trait_decl_node, "".into());
        }
    };
    let mut methods_and_indexes = vec![];
    // insert method declarations into the graph
    for fn_decl in methods {
        let fn_decl_entry_node = graph.add_node(ControlFlowGraphNode::MethodDeclaration {
            span: fn_decl.span.clone(),
            method_name: fn_decl.name.clone(),
        });
        graph.add_edge(entry_node, fn_decl_entry_node, "".into());
        // connect the impl declaration node to the functions themselves, as all trait functions are
        // public if the trait is in scope
        connect_typed_fn_decl(
            fn_decl,
            graph,
            fn_decl_entry_node,
            fn_decl.span.clone(),
            None,
            tree_type,
        )?;
        methods_and_indexes.push((fn_decl.name.clone(), fn_decl_entry_node));
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
    decl: &TypedTraitDeclaration,
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
    decl: &TypedAbiDeclaration,
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
/// For an enum declaration, we want to make a declaration node for every individual enum
/// variant. When a variant is constructed, we can point an edge at that variant. This way,
/// we can see clearly, and thusly warn, when individual variants are not ever constructed.
fn connect_enum_declaration(
    enum_decl: &TypedEnumDeclaration,
    graph: &mut ControlFlowGraph,
    entry_node: NodeIndex,
) {
    // keep a mapping of each variant
    for variant in &enum_decl.variants {
        let variant_index = graph.add_node(ControlFlowGraphNode::from_enum_variant(
            variant,
            enum_decl.visibility != Visibility::Private,
        ));

        graph.namespace.insert_enum(
            enum_decl.name.clone(),
            entry_node,
            variant.name.clone(),
            variant_index,
        );
    }
}

/// When connecting a function declaration, we are inserting a new root node into the graph that
/// has no entry points, since it is just a declaration.
/// When something eventually calls it, it gets connected to the declaration.
fn connect_typed_fn_decl(
    fn_decl: &TypedFunctionDeclaration,
    graph: &mut ControlFlowGraph,
    entry_node: NodeIndex,
    span: Span,
    exit_node: Option<NodeIndex>,
    tree_type: &TreeType,
) -> Result<(), CompileError> {
    let fn_exit_node = graph.add_node(format!("\"{}\" fn exit", fn_decl.name.as_str()).into());
    let (_exit_nodes, _exit_node) = depth_first_insertion_code_block(
        &fn_decl.body,
        graph,
        &[entry_node],
        Some(fn_exit_node),
        tree_type,
    )?;
    if let Some(exit_node) = exit_node {
        graph.add_edge(fn_exit_node, exit_node, "".into());
    }

    // not sure how correct it is to default to Unit here...
    // I think types should all be resolved by now.
    let ty =
        resolve_type(fn_decl.return_type, &span).unwrap_or_else(|_| TypeInfo::Tuple(Vec::new()));

    let namespace_entry = FunctionNamespaceEntry {
        entry_point: entry_node,
        exit_point: fn_exit_node,
        return_type: ty,
    };

    graph
        .namespace
        .insert_function(fn_decl.name.clone(), namespace_entry);
    Ok(())
}

fn depth_first_insertion_code_block(
    node_content: &TypedCodeBlock,
    graph: &mut ControlFlowGraph,
    leaves: &[NodeIndex],
    exit_node: Option<NodeIndex>,
    tree_type: &TreeType,
) -> Result<(Vec<NodeIndex>, Option<NodeIndex>), CompileError> {
    let mut leaves = leaves.to_vec();
    let mut exit_node = exit_node;
    for node in node_content.contents.iter() {
        let (this_node, l_exit_node) = connect_node(node, graph, &leaves, exit_node, tree_type)?;
        leaves = this_node;
        exit_node = l_exit_node;
    }
    Ok((leaves, exit_node))
}

/// connects any inner parts of an expression to the graph
/// note the main expression node has already been inserted
fn connect_expression(
    expr_variant: &TypedExpressionVariant,
    graph: &mut ControlFlowGraph,
    leaves: &[NodeIndex],
    exit_node: Option<NodeIndex>,
    label: &'static str,
    tree_type: &TreeType,
    _expression_span: Span,
) -> Result<Vec<NodeIndex>, CompileError> {
    use TypedExpressionVariant::*;
    match expr_variant {
        FunctionApplication {
            name, arguments, ..
        } => {
            let mut is_external = false;
            // find the function in the namespace
            let (fn_entrypoint, fn_exit_point) = graph
                .namespace
                .get_function(&name.suffix)
                .cloned()
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
            for leaf in leaves {
                graph.add_edge(*leaf, fn_entrypoint, label.into());
            }
            // we evaluate every one of the function arguments
            let mut current_leaf = vec![fn_entrypoint];
            for (_name, arg) in arguments {
                current_leaf = connect_expression(
                    &arg.expression,
                    graph,
                    &current_leaf,
                    exit_node,
                    "arg eval",
                    tree_type,
                    arg.clone().span,
                )?;
            }
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
                &lhs.expression,
                graph,
                leaves,
                exit_node,
                "",
                tree_type,
                lhs.span.clone(),
            )?;
            let rhs_expr = connect_expression(
                &rhs.expression,
                graph,
                leaves,
                exit_node,
                "",
                tree_type,
                rhs.span.clone(),
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
                enum_decl,
                contents,
                variant_name,
                graph,
                leaves,
                exit_node,
                tree_type,
            )
        }
        IfExp {
            condition,
            then,
            r#else,
        } => {
            let condition_expr = connect_expression(
                &(*condition).expression,
                graph,
                leaves,
                exit_node,
                "",
                tree_type,
                (*condition).span.clone(),
            )?;
            let then_expr = connect_expression(
                &(*then).expression,
                graph,
                &condition_expr,
                exit_node,
                "then branch",
                tree_type,
                (*then).span.clone(),
            )?;

            let else_expr = if let Some(else_expr) = r#else {
                connect_expression(
                    &(*else_expr).expression,
                    graph,
                    &condition_expr,
                    exit_node,
                    "else branch",
                    tree_type,
                    else_expr.clone().span,
                )?
            } else {
                vec![]
            };

            Ok([then_expr, else_expr].concat())
        }
        CodeBlock(a @ TypedCodeBlock { .. }) => {
            connect_code_block(a, graph, leaves, exit_node, tree_type)
        }
        StructExpression {
            struct_name,
            fields,
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

            let mut current_leaf = vec![entry];
            // for every field, connect its expression
            for TypedStructExpressionField { value, .. } in fields {
                current_leaf = connect_expression(
                    &value.expression,
                    graph,
                    &current_leaf,
                    exit_node,
                    "struct field instantiation",
                    tree_type,
                    value.clone().span,
                )?;
            }

            // connect the final field to the exit
            for leaf in current_leaf {
                graph.add_edge(leaf, exit, "".into());
            }
            Ok(vec![exit])
        }
        StructFieldAccess {
            field_to_access,
            resolved_type_of_parent,
            ..
        } => {
            let resolved_type_of_parent =
                resolve_type(*resolved_type_of_parent, &field_to_access.span)
                    .unwrap_or_else(|_| TypeInfo::Tuple(Vec::new()));

            assert!(matches!(resolved_type_of_parent, TypeInfo::Struct { .. }));
            let resolved_type_of_parent = match resolved_type_of_parent {
                TypeInfo::Struct { name, .. } => name,
                _ => panic!("Called subfield on a non-struct"),
            };
            let field_name = &field_to_access.name;
            // find the struct field index in the namespace
            let field_ix = match graph
                .namespace
                .find_struct_field_idx(resolved_type_of_parent.as_str(), field_name.as_str())
            {
                Some(ix) => *ix,
                None => graph.add_node("external struct".into()),
            };

            let this_ix = graph.add_node(
                format!(
                    "Struct field access: {}.{}",
                    resolved_type_of_parent, field_name
                )
                .into(),
            );
            for leaf in leaves {
                graph.add_edge(*leaf, this_ix, "".into());
            }
            graph.add_edge(this_ix, field_ix, "".into());
            Ok(vec![this_ix])
        }
        AsmExpression { .. } => {
            let asm_node = graph.add_node("Inline asm".into());
            for leaf in leaves {
                graph.add_edge(*leaf, asm_node, "".into());
            }
            Ok(vec![asm_node])
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
                    &value.expression,
                    graph,
                    &current_leaf,
                    exit_node,
                    "tuple field instantiation",
                    tree_type,
                    value.clone().span,
                )?;
            }

            // connect the final field to the exit
            for leaf in current_leaf {
                graph.add_edge(leaf, exit, "".into());
            }
            Ok(vec![exit])
        }
        AbiCast { address, .. } => connect_expression(
            &address.expression,
            graph,
            leaves,
            exit_node,
            "abi cast address",
            tree_type,
            address.span.clone(),
        ),
        Array { contents } => {
            let nodes = contents
                .iter()
                .map(|elem| {
                    connect_expression(
                        &elem.expression,
                        graph,
                        leaves,
                        exit_node,
                        "",
                        tree_type,
                        elem.span.clone(),
                    )
                })
                .collect::<Result<Vec<_>, _>>()?;
            Ok(nodes.concat())
        }
        ArrayIndex { prefix, index } => {
            let prefix_idx = connect_expression(
                &prefix.expression,
                graph,
                leaves,
                exit_node,
                "",
                tree_type,
                prefix.span.clone(),
            )?;
            let index_idx = connect_expression(
                &index.expression,
                graph,
                leaves,
                exit_node,
                "",
                tree_type,
                index.span.clone(),
            )?;
            Ok([prefix_idx, index_idx].concat())
        }
        IfLet {
            expr, then, r#else, ..
        } => {
            let leaves = connect_expression(
                &expr.expression,
                graph,
                leaves,
                exit_node,
                "if let expr",
                tree_type,
                expr.span.clone(),
            )?;
            let then_expr = connect_code_block(then, graph, &leaves, exit_node, tree_type)?;

            let else_expr = if let Some(else_expr) = r#else {
                connect_expression(
                    &else_expr.expression,
                    graph,
                    &leaves,
                    exit_node,
                    "if let: else branch",
                    tree_type,
                    (**else_expr).span.clone(),
                )?
            } else {
                vec![]
            };
            Ok([then_expr, else_expr].concat())
        }
        TupleElemAccess { prefix, .. } => {
            let prefix_idx = connect_expression(
                &prefix.expression,
                graph,
                leaves,
                exit_node,
                "",
                tree_type,
                prefix.span.clone(),
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
        TypeProperty { .. } => {
            let node = graph.add_node("Type Property".into());
            for leaf in leaves {
                graph.add_edge(*leaf, node, "".into());
            }
            Ok(vec![node])
        }
        SizeOfValue { expr } => {
            let expr = connect_expression(
                &(*expr).expression,
                graph,
                leaves,
                exit_node,
                "size_of",
                tree_type,
                expr.span.clone(),
            )?;
            Ok(expr)
        }
        AbiName(abi_name) => {
            if let crate::type_engine::AbiName::Known(abi_name) = abi_name {
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
    }
}

fn connect_code_block(
    block: &TypedCodeBlock,
    graph: &mut ControlFlowGraph,
    leaves: &[NodeIndex],
    exit_node: Option<NodeIndex>,
    tree_type: &TreeType,
) -> Result<Vec<NodeIndex>, CompileError> {
    let contents = &block.contents;
    let block_entry = graph.add_node("Code block entry".into());
    for leaf in leaves {
        graph.add_edge(*leaf, block_entry, "".into());
    }
    let mut current_leaf = vec![block_entry];
    for node in contents {
        current_leaf = connect_node(node, graph, &current_leaf, exit_node, tree_type)?.0;
    }

    let block_exit = graph.add_node("Code block exit".into());
    for leaf in current_leaf {
        graph.add_edge(leaf, block_exit, "".into());
    }
    Ok(vec![block_exit])
}

fn connect_enum_instantiation(
    enum_decl: &TypedEnumDeclaration,
    contents: &Option<Box<TypedExpression>>,
    variant_name: &Ident,
    graph: &mut ControlFlowGraph,
    leaves: &[NodeIndex],
    exit_node: Option<NodeIndex>,
    tree_type: &TreeType,
) -> Result<Vec<NodeIndex>, CompileError> {
    let enum_name = &enum_decl.name;
    let (decl_ix, variant_index) = graph
        .namespace
        .find_enum_variant_index(enum_name, variant_name)
        .unwrap_or_else(|| {
            let node_idx = graph.add_node(
                format!(
                    "extern enum {}::{}",
                    enum_name.as_str(),
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
            &instantiator.expression,
            graph,
            &[enum_instantiation_entry_idx],
            exit_node,
            "",
            tree_type,
            enum_decl.span.clone(),
        )?;
        for leaf in instantiator_contents {
            graph.add_edge(leaf, enum_instantiation_exit_idx, "".into());
        }
    }

    graph.add_edge(decl_ix, variant_index, "".into());
    graph.add_edge(variant_index, enum_instantiation_exit_idx, "".into());

    Ok(vec![enum_instantiation_exit_idx])
}

/// Given a `TypedAstNode` that we know is not reached in the graph, construct a warning
/// representing its unreached status. For example, we want to say "this function is never called"
/// if the node is a function declaration, but "this trait is never used" if it is a trait
/// declaration.
fn construct_dead_code_warning_from_node(node: &TypedAstNode) -> Option<CompileWarning> {
    Some(match node {
        // if this is a function, struct, or trait declaration that is never called, then it is dead
        // code.
        TypedAstNode {
            content:
                TypedAstNodeContent::Declaration(TypedDeclaration::FunctionDeclaration(
                    TypedFunctionDeclaration { .. },
                )),
            span,
            ..
        } => CompileWarning {
            span: span.clone(),
            warning_content: Warning::DeadFunctionDeclaration,
        },
        TypedAstNode {
            content: TypedAstNodeContent::Declaration(TypedDeclaration::StructDeclaration { .. }),
            span,
        } => CompileWarning {
            span: span.clone(),
            warning_content: Warning::DeadStructDeclaration,
        },
        TypedAstNode {
            content:
                TypedAstNodeContent::Declaration(TypedDeclaration::TraitDeclaration(
                    TypedTraitDeclaration { name, .. },
                )),
            ..
        } => CompileWarning {
            span: name.span().clone(),
            warning_content: Warning::DeadTrait,
        },
        TypedAstNode {
            content: TypedAstNodeContent::Declaration(TypedDeclaration::ImplTrait { methods, .. }),
            ..
        } if methods.is_empty() => return None,
        TypedAstNode {
            content: TypedAstNodeContent::Declaration(TypedDeclaration::AbiDeclaration { .. }),
            ..
        } => return None,
        TypedAstNode {
            content: TypedAstNodeContent::Declaration(..),
            span,
        } => CompileWarning {
            span: span.clone(),
            warning_content: Warning::DeadDeclaration,
        },
        // otherwise, this is unreachable.
        TypedAstNode { span, .. } => CompileWarning {
            span: span.clone(),
            warning_content: Warning::UnreachableCode,
        },
    })
}

fn connect_storage_declaration(
    decl: &TypedStorageDeclaration,
    graph: &mut ControlFlowGraph,
    _entry_node: NodeIndex,
    _tree_type: &TreeType,
) {
    let TypedStorageDeclaration { fields, .. } = decl;

    let field_nodes = fields
        .iter()
        .map(|field| (field.clone(), graph.add_node(field.into())))
        .collect::<Vec<_>>();

    graph.namespace.insert_storage(field_nodes);
}
