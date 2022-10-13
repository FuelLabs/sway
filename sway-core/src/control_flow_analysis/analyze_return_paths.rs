//! This is the flow graph, a graph which contains edges that represent possible steps of program
//! execution.

use crate::{
    control_flow_analysis::*,
    declaration_engine::declaration_engine::{de_get_function, de_get_impl_trait},
    language::{ty, CallPath},
    semantic_analysis::*,
    type_system::*,
};
use petgraph::prelude::NodeIndex;
use sway_error::error::CompileError;
use sway_types::{ident::Ident, span::Span, Spanned};

impl ControlFlowGraph {
    pub(crate) fn construct_return_path_graph(
        module_nodes: &[TyAstNode],
    ) -> Result<Self, CompileError> {
        let mut graph = ControlFlowGraph::default();
        // do a depth first traversal and cover individual inner ast nodes
        let mut leaves = vec![];
        for ast_entrypoint in module_nodes {
            let l_leaves = connect_node(ast_entrypoint, &mut graph, &leaves)?.0;
            if let NodeConnection::NextStep(nodes) = l_leaves {
                leaves = nodes;
            }
        }
        Ok(graph)
    }

    /// This function looks through the control flow graph and ensures that all paths that are
    /// required to return a value do, indeed, return a value of the correct type.
    /// It does this by checking every function declaration in both the methods namespace
    /// and the functions namespace and validating that all paths leading to the function exit node
    /// return the same type. Additionally, if a function has a return type, all paths must indeed
    /// lead to the function exit node.
    pub(crate) fn analyze_return_paths(&self) -> Vec<CompileError> {
        let mut errors = vec![];
        for (
            name,
            FunctionNamespaceEntry {
                entry_point,
                exit_point,
                return_type,
            },
        ) in &self.namespace.function_namespace
        {
            // For every node connected to the entry point
            errors.append(&mut self.ensure_all_paths_reach_exit(
                *entry_point,
                *exit_point,
                name,
                return_type,
            ));
        }
        errors
    }

    fn ensure_all_paths_reach_exit(
        &self,
        entry_point: EntryPoint,
        exit_point: ExitPoint,
        function_name: &Ident,
        return_ty: &TypeInfo,
    ) -> Vec<CompileError> {
        let mut rovers = vec![entry_point];
        let mut errors = vec![];
        let mut max_iterations = 50;
        while !rovers.is_empty() && rovers[0] != exit_point && max_iterations > 0 {
            max_iterations -= 1;
            rovers.retain(|idx| *idx != exit_point);
            let mut next_rovers = vec![];
            let mut last_discovered_span;
            for rover in rovers {
                last_discovered_span = match &self.graph[rover] {
                    ControlFlowGraphNode::ProgramNode(node) => Some(node.span.clone()),
                    ControlFlowGraphNode::MethodDeclaration { span, .. } => Some(span.clone()),
                    _ => None,
                };

                let mut neighbors = self
                    .graph
                    .neighbors_directed(rover, petgraph::Direction::Outgoing)
                    .collect::<Vec<_>>();
                if neighbors.is_empty() && !return_ty.is_unit() {
                    let span = match last_discovered_span {
                        Some(ref o) => o.clone(),
                        None => {
                            errors.push(CompileError::Internal(
                                "Attempted to construct return path error \
                                    but no source span was found.",
                                Span::dummy(),
                            ));
                            return errors;
                        }
                    };
                    errors.push(CompileError::PathDoesNotReturn {
                        // TODO: unwrap_to_node is a shortcut. In reality, the graph type should be
                        // different. To save some code duplication,
                        span,
                        function_name: function_name.clone(),
                        ty: return_ty.to_string(),
                    });
                }
                next_rovers.append(&mut neighbors);
            }
            rovers = next_rovers;
        }

        errors
    }
}

/// The resulting edges from connecting a node to the graph.
enum NodeConnection {
    /// This represents a node that steps on to the next node.
    NextStep(Vec<NodeIndex>),
    /// This represents a return or implicit return node, which aborts the stepwise flow.
    Return(NodeIndex),
}

fn connect_node(
    node: &TyAstNode,
    graph: &mut ControlFlowGraph,
    leaves: &[NodeIndex],
) -> Result<(NodeConnection, ReturnStatementNodes), CompileError> {
    let span = node.span.clone();
    match &node.content {
        TyAstNodeContent::Expression(ty::TyExpression {
            expression: ty::TyExpressionVariant::Return(..),
            ..
        })
        | TyAstNodeContent::ImplicitReturnExpression(_) => {
            let this_index = graph.add_node(node.into());
            for leaf_ix in leaves {
                graph.add_edge(*leaf_ix, this_index, "".into());
            }
            Ok((NodeConnection::Return(this_index), vec![]))
        }
        TyAstNodeContent::Expression(ty::TyExpression {
            expression: ty::TyExpressionVariant::WhileLoop { body, .. },
            ..
        }) => {
            // This is very similar to the dead code analysis for a while loop.
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

            // We need to dig into the body of the while loop in case there is a break or a
            // continue at some level.
            let (l_leaves, inner_returns) = depth_first_insertion_code_block(body, graph, &leaves)?;

            // insert edges from end of block back to beginning of it
            for leaf in &l_leaves {
                graph.add_edge(*leaf, entry, "loop repeats".into());
            }

            leaves = l_leaves;
            for leaf in leaves {
                graph.add_edge(leaf, while_loop_exit, "".into());
            }

            Ok((
                NodeConnection::NextStep(vec![while_loop_exit]),
                inner_returns,
            ))
        }
        TyAstNodeContent::Expression(ty::TyExpression { .. }) => {
            let entry = graph.add_node(node.into());
            // insert organizational dominator node
            // connected to all current leaves
            for leaf in leaves {
                graph.add_edge(*leaf, entry, "".into());
            }
            Ok((NodeConnection::NextStep(vec![entry]), vec![]))
        }
        TyAstNodeContent::SideEffect => Ok((NodeConnection::NextStep(leaves.to_vec()), vec![])),
        TyAstNodeContent::Declaration(decl) => Ok((
            NodeConnection::NextStep(connect_declaration(node, decl, graph, span, leaves)?),
            vec![],
        )),
    }
}

fn connect_declaration(
    node: &TyAstNode,
    decl: &ty::TyDeclaration,
    graph: &mut ControlFlowGraph,
    span: Span,
    leaves: &[NodeIndex],
) -> Result<Vec<NodeIndex>, CompileError> {
    use ty::TyDeclaration::*;
    match decl {
        TraitDeclaration(_)
        | AbiDeclaration(_)
        | StructDeclaration(_)
        | EnumDeclaration(_)
        | StorageDeclaration(_)
        | GenericTypeForFunctionScope { .. } => Ok(leaves.to_vec()),
        VariableDeclaration(_) | ConstantDeclaration(_) => {
            let entry_node = graph.add_node(node.into());
            for leaf in leaves {
                graph.add_edge(*leaf, entry_node, "".into());
            }
            Ok(vec![entry_node])
        }
        FunctionDeclaration(decl_id) => {
            let fn_decl = de_get_function(decl_id.clone(), &decl.span())?;
            let entry_node = graph.add_node(node.into());
            for leaf in leaves {
                graph.add_edge(*leaf, entry_node, "".into());
            }
            connect_typed_fn_decl(&fn_decl, graph, entry_node, span)?;
            Ok(leaves.to_vec())
        }
        ImplTrait(decl_id) => {
            let TyImplTrait {
                trait_name,
                methods,
                ..
            } = de_get_impl_trait(decl_id.clone(), &span)?;
            let entry_node = graph.add_node(node.into());
            for leaf in leaves {
                graph.add_edge(*leaf, entry_node, "".into());
            }
            connect_impl_trait(&trait_name, graph, &methods, entry_node)?;
            Ok(leaves.to_vec())
        }
        ErrorRecovery => Ok(leaves.to_vec()),
    }
}

/// Implementations of traits are top-level things that are not conditional, so
/// we insert an edge from the function's starting point to the declaration to show
/// that the declaration was indeed at some point implemented.
/// Additionally, we insert the trait's methods into the method namespace in order to
/// track which exact methods are dead code.
fn connect_impl_trait(
    trait_name: &CallPath,
    graph: &mut ControlFlowGraph,
    methods: &[TyFunctionDeclaration],
    entry_node: NodeIndex,
) -> Result<(), CompileError> {
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
        connect_typed_fn_decl(fn_decl, graph, fn_decl_entry_node, fn_decl.span.clone())?;
        methods_and_indexes.push((fn_decl.name.clone(), fn_decl_entry_node));
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

/// When connecting a function declaration, we are inserting a new root node into the graph that
/// has no entry points, since it is just a declaration.
/// When something eventually calls it, it gets connected to the declaration.
fn connect_typed_fn_decl(
    fn_decl: &TyFunctionDeclaration,
    graph: &mut ControlFlowGraph,
    entry_node: NodeIndex,
    _span: Span,
) -> Result<(), CompileError> {
    let fn_exit_node = graph.add_node(format!("\"{}\" fn exit", fn_decl.name.as_str()).into());
    let return_nodes = depth_first_insertion_code_block(&fn_decl.body, graph, &[entry_node])?.0;
    for node in return_nodes {
        graph.add_edge(node, fn_exit_node, "return".into());
    }

    let namespace_entry = FunctionNamespaceEntry {
        entry_point: entry_node,
        exit_point: fn_exit_node,
        return_type: to_typeinfo(fn_decl.return_type, &fn_decl.return_type_span)
            .unwrap_or_else(|_| TypeInfo::Tuple(Vec::new())),
    };
    graph
        .namespace
        .insert_function(fn_decl.name.clone(), namespace_entry);
    Ok(())
}

type ReturnStatementNodes = Vec<NodeIndex>;

fn depth_first_insertion_code_block(
    node_content: &TyCodeBlock,
    graph: &mut ControlFlowGraph,
    leaves: &[NodeIndex],
) -> Result<(ReturnStatementNodes, Vec<NodeIndex>), CompileError> {
    let mut leaves = leaves.to_vec();
    let mut return_nodes = vec![];
    for node in node_content.contents.iter() {
        let (this_node, inner_returns) = connect_node(node, graph, &leaves)?;
        match this_node {
            NodeConnection::NextStep(nodes) => leaves = nodes,
            NodeConnection::Return(node) => {
                return_nodes.push(node);
            }
        }
        return_nodes.extend(inner_returns);
    }
    Ok((return_nodes, leaves))
}
