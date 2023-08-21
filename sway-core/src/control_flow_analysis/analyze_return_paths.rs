//! This is the flow graph, a graph which contains edges that represent possible steps of program
//! execution.

use crate::{
    control_flow_analysis::*,
    language::{
        ty::{self, TyImplItem},
        CallPath,
    },
    type_system::*,
    Engines,
};
use petgraph::prelude::NodeIndex;
use sway_error::error::CompileError;
use sway_types::{ident::Ident, span::Span, IdentUnique};

impl<'cfg> ControlFlowGraph<'cfg> {
    pub(crate) fn construct_return_path_graph<'eng: 'cfg>(
        engines: &'eng Engines,
        module_nodes: &[ty::TyAstNode],
    ) -> Result<Self, Vec<CompileError>> {
        let mut errors = vec![];

        let mut graph = ControlFlowGraph::default();
        // do a depth first traversal and cover individual inner ast nodes
        let mut leaves = vec![];
        for ast_entrypoint in module_nodes {
            match connect_node(engines, ast_entrypoint, &mut graph, &leaves) {
                Ok(NodeConnection::NextStep(nodes)) => {
                    leaves = nodes;
                }
                Ok(_) => {}
                Err(mut e) => {
                    errors.append(&mut e);
                }
            }
        }

        if !errors.is_empty() {
            Err(errors)
        } else {
            Ok(graph)
        }
    }

    /// This function looks through the control flow graph and ensures that all paths that are
    /// required to return a value do, indeed, return a value of the correct type.
    /// It does this by checking every function declaration in both the methods namespace
    /// and the functions namespace and validating that all paths leading to the function exit node
    /// return the same type. Additionally, if a function has a return type, all paths must indeed
    /// lead to the function exit node.
    pub(crate) fn analyze_return_paths(&self, engines: &Engines) -> Vec<CompileError> {
        let mut errors = vec![];
        for (
            (name, _sig),
            FunctionNamespaceEntry {
                entry_point,
                exit_point,
                return_type,
            },
        ) in &self.namespace.function_namespace
        {
            // For every node connected to the entry point
            errors.append(&mut self.ensure_all_paths_reach_exit(
                engines,
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
        engines: &Engines,
        entry_point: EntryPoint,
        exit_point: ExitPoint,
        function_name: &IdentUnique,
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
                    ControlFlowGraphNode::ProgramNode { node, .. } => Some(node.span.clone()),
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
                    let function_name: Ident = function_name.into();
                    errors.push(CompileError::PathDoesNotReturn {
                        // TODO: unwrap_to_node is a shortcut. In reality, the graph type should be
                        // different. To save some code duplication,
                        span,
                        function_name: function_name.clone(),
                        ty: engines.help_out(return_ty).to_string(),
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

fn connect_node<'eng: 'cfg, 'cfg>(
    engines: &'eng Engines,
    node: &ty::TyAstNode,
    graph: &mut ControlFlowGraph<'cfg>,
    leaves: &[NodeIndex],
) -> Result<NodeConnection, Vec<CompileError>> {
    match &node.content {
        ty::TyAstNodeContent::Expression(ty::TyExpression {
            expression: ty::TyExpressionVariant::Return(..),
            ..
        })
        | ty::TyAstNodeContent::ImplicitReturnExpression(_) => {
            let this_index = graph.add_node(ControlFlowGraphNode::from_node(node));
            for leaf_ix in leaves {
                graph.add_edge(*leaf_ix, this_index, "".into());
            }
            Ok(NodeConnection::Return(this_index))
        }
        ty::TyAstNodeContent::Expression(ty::TyExpression {
            expression: ty::TyExpressionVariant::WhileLoop { .. },
            ..
        }) => {
            // An abridged version of the dead code analysis for a while loop
            // since we don't really care about what the loop body contains when detecting
            // divergent paths
            let node = graph.add_node(ControlFlowGraphNode::from_node(node));
            for leaf in leaves {
                graph.add_edge(*leaf, node, "while loop entry".into());
            }
            Ok(NodeConnection::NextStep(vec![node]))
        }
        ty::TyAstNodeContent::Expression(ty::TyExpression { .. }) => {
            let entry = graph.add_node(ControlFlowGraphNode::from_node(node));
            // insert organizational dominator node
            // connected to all current leaves
            for leaf in leaves {
                graph.add_edge(*leaf, entry, "".into());
            }
            Ok(NodeConnection::NextStep(vec![entry]))
        }
        ty::TyAstNodeContent::SideEffect(_) => Ok(NodeConnection::NextStep(leaves.to_vec())),
        ty::TyAstNodeContent::Declaration(decl) => Ok(NodeConnection::NextStep(
            connect_declaration(engines, node, decl, graph, leaves)?,
        )),
        ty::TyAstNodeContent::Error(_, _) => Ok(NodeConnection::NextStep(vec![])),
    }
}

fn connect_declaration<'eng: 'cfg, 'cfg>(
    engines: &'eng Engines,
    node: &ty::TyAstNode,
    decl: &ty::TyDecl,
    graph: &mut ControlFlowGraph<'cfg>,
    leaves: &[NodeIndex],
) -> Result<Vec<NodeIndex>, Vec<CompileError>> {
    let decl_engine = engines.de();
    match decl {
        ty::TyDecl::TraitDecl(_)
        | ty::TyDecl::AbiDecl(_)
        | ty::TyDecl::StructDecl(_)
        | ty::TyDecl::EnumDecl(_)
        | ty::TyDecl::EnumVariantDecl(_)
        | ty::TyDecl::StorageDecl(_)
        | ty::TyDecl::TypeAliasDecl(_)
        | ty::TyDecl::TypeDecl(_)
        | ty::TyDecl::GenericTypeForFunctionScope(_) => Ok(leaves.to_vec()),
        ty::TyDecl::VariableDecl(_) | ty::TyDecl::ConstantDecl(_) => {
            let entry_node = graph.add_node(ControlFlowGraphNode::from_node(node));
            for leaf in leaves {
                graph.add_edge(*leaf, entry_node, "".into());
            }
            Ok(vec![entry_node])
        }
        ty::TyDecl::FunctionDecl(ty::FunctionDecl { decl_id, .. }) => {
            let fn_decl = decl_engine.get_function(decl_id);
            let entry_node = graph.add_node(ControlFlowGraphNode::from_node(node));
            for leaf in leaves {
                graph.add_edge(*leaf, entry_node, "".into());
            }
            connect_typed_fn_decl(engines, &fn_decl, graph, entry_node)?;
            Ok(leaves.to_vec())
        }
        ty::TyDecl::ImplTrait(ty::ImplTrait { decl_id, .. }) => {
            let ty::TyImplTrait {
                trait_name, items, ..
            } = decl_engine.get_impl_trait(decl_id);
            let entry_node = graph.add_node(ControlFlowGraphNode::from_node(node));
            for leaf in leaves {
                graph.add_edge(*leaf, entry_node, "".into());
            }

            connect_impl_trait(engines, &trait_name, graph, &items, entry_node)?;
            Ok(leaves.to_vec())
        }
        ty::TyDecl::ErrorRecovery(..) => Ok(leaves.to_vec()),
    }
}

/// Implementations of traits are top-level things that are not conditional, so
/// we insert an edge from the function's starting point to the declaration to show
/// that the declaration was indeed at some point implemented.
/// Additionally, we insert the trait's methods into the method namespace in order to
/// track which exact methods are dead code.
fn connect_impl_trait<'eng: 'cfg, 'cfg>(
    engines: &'eng Engines,
    trait_name: &CallPath,
    graph: &mut ControlFlowGraph<'cfg>,
    items: &[TyImplItem],
    entry_node: NodeIndex,
) -> Result<(), Vec<CompileError>> {
    let decl_engine = engines.de();
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
                graph.add_edge(entry_node, fn_decl_entry_node, "".into());
                // connect the impl declaration node to the functions themselves, as all trait functions are
                // public if the trait is in scope
                connect_typed_fn_decl(engines, &fn_decl, graph, fn_decl_entry_node)?;
                methods_and_indexes.push((fn_decl.name.clone(), fn_decl_entry_node));
            }
            TyImplItem::Constant(_const_decl) => {}
            TyImplItem::Type(_type_decl) => {}
        }
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
fn connect_typed_fn_decl<'eng: 'cfg, 'cfg>(
    engines: &'eng Engines,
    fn_decl: &ty::TyFunctionDecl,
    graph: &mut ControlFlowGraph<'cfg>,
    entry_node: NodeIndex,
) -> Result<(), Vec<CompileError>> {
    let type_engine = engines.te();
    let fn_exit_node = graph.add_node(format!("\"{}\" fn exit", fn_decl.name.as_str()).into());
    let return_nodes =
        depth_first_insertion_code_block(engines, &fn_decl.body, graph, &[entry_node])?;
    for node in return_nodes {
        graph.add_edge(node, fn_exit_node, "return".into());
    }

    let namespace_entry = FunctionNamespaceEntry {
        entry_point: entry_node,
        exit_point: fn_exit_node,
        return_type: type_engine
            .to_typeinfo(fn_decl.return_type.type_id, &fn_decl.return_type.span)
            .unwrap_or_else(|_| TypeInfo::Tuple(Vec::new())),
    };
    graph.namespace.insert_function(fn_decl, namespace_entry);
    Ok(())
}

type ReturnStatementNodes = Vec<NodeIndex>;

fn depth_first_insertion_code_block<'eng: 'cfg, 'cfg>(
    engines: &'eng Engines,
    node_content: &ty::TyCodeBlock,
    graph: &mut ControlFlowGraph<'cfg>,
    leaves: &[NodeIndex],
) -> Result<ReturnStatementNodes, Vec<CompileError>> {
    let mut errors = vec![];

    let mut leaves = leaves.to_vec();
    let mut return_nodes = vec![];
    for node in node_content.contents.iter() {
        match connect_node(engines, node, graph, &leaves) {
            Ok(this_node) => match this_node {
                NodeConnection::NextStep(nodes) => leaves = nodes,
                NodeConnection::Return(node) => {
                    return_nodes.push(node);
                }
            },
            Err(mut e) => errors.append(&mut e),
        }
    }

    if !errors.is_empty() {
        Err(errors)
    } else {
        Ok(return_nodes)
    }
}
