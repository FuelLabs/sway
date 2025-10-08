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
use sway_types::{ident::Ident, span::Span, IdentUnique, Spanned};

impl<'cfg> ControlFlowGraph<'cfg> {
    pub(crate) fn construct_return_path_graph<'eng: 'cfg>(
        engines: &'eng Engines,
        module_nodes: &[ty::TyAstNode],
    ) -> Result<Self, Vec<CompileError>> {
        let mut errors = vec![];

        let mut graph = ControlFlowGraph::new(engines);
        // do a depth first traversal and cover individual inner ast nodes
        let mut leaf_opt = None;
        for ast_entrypoint in module_nodes {
            match connect_node(engines, ast_entrypoint, &mut graph, leaf_opt) {
                Ok(NodeConnection::NextStep(node_opt)) => {
                    leaf_opt = node_opt;
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

    /// Traverses the spine of a function to ensure that it does return if a return value is
    /// expected.  The spine of the function does not include branches such as if-then-elses and
    /// loops. Those branches are ignored, and a branching expression is represented as a single
    /// node in the graph. The analysis continues once the branches join again.  This means that the
    /// spine is linear, so every node has at most one outgoing edge. The graph is assumed to have
    /// been constructed this way.
    fn ensure_all_paths_reach_exit(
        &self,
        engines: &Engines,
        entry_point: EntryPoint,
        exit_point: ExitPoint,
        function_name: &IdentUnique,
        return_ty: &TypeInfo,
    ) -> Vec<CompileError> {
        let mut rover = entry_point;
        let mut errors = vec![];

        while rover != exit_point {
            let neighbors = self
                .graph
                .neighbors_directed(rover, petgraph::Direction::Outgoing)
                .collect::<Vec<_>>();

            // The graph is supposed to be a single path, so at most one outgoing neighbor is allowed.
            assert!(neighbors.len() <= 1);

            if neighbors.is_empty() {
                if !return_ty.is_unit() {
                    // A return is expected, but none is found. Report an error.
                    let span = match &self.graph[rover] {
                        ControlFlowGraphNode::ProgramNode { node, .. } => node.span.clone(),
                        ControlFlowGraphNode::MethodDeclaration { span, .. } => span.clone(),
                        _ => {
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
                        span,
                        function_name: function_name.clone(),
                        ty: engines.help_out(return_ty).to_string(),
                    });
                }

                // No further neighbors, so we're done.
                break;
            }

            rover = neighbors[0];
        }

        errors
    }
}

/// The resulting edges from connecting a node to the graph.
enum NodeConnection {
    /// This represents a node that steps on to the next node.
    NextStep(Option<NodeIndex>),
    /// This represents a node which aborts the stepwise flow.
    /// Such nodes are:
    /// - return expressions,
    /// - implicit returns,
    /// - panic expressions.
    Return(NodeIndex),
}

fn connect_node<'eng: 'cfg, 'cfg>(
    engines: &'eng Engines,
    node: &ty::TyAstNode,
    graph: &mut ControlFlowGraph<'cfg>,
    leaf_opt: Option<NodeIndex>,
) -> Result<NodeConnection, Vec<CompileError>> {
    match &node.content {
        ty::TyAstNodeContent::Expression(ty::TyExpression {
            expression: ty::TyExpressionVariant::Return(..),
            ..
        })
        | ty::TyAstNodeContent::Expression(ty::TyExpression {
            expression: ty::TyExpressionVariant::ImplicitReturn(..),
            ..
        })
        | ty::TyAstNodeContent::Expression(ty::TyExpression {
            expression: ty::TyExpressionVariant::Panic(..),
            ..
        }) => {
            let this_index = graph.add_node(ControlFlowGraphNode::from_node(node));
            if let Some(leaf_ix) = leaf_opt {
                graph.add_edge(leaf_ix, this_index, "".into());
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
            if let Some(leaf) = leaf_opt {
                graph.add_edge(leaf, node, "while loop entry".into());
            }
            Ok(NodeConnection::NextStep(Some(node)))
        }
        ty::TyAstNodeContent::Expression(ty::TyExpression { .. }) => {
            let entry = graph.add_node(ControlFlowGraphNode::from_node(node));
            // insert organizational dominator node
            // connected to all current leaves
            if let Some(leaf) = leaf_opt {
                graph.add_edge(leaf, entry, "".into());
            }
            Ok(NodeConnection::NextStep(Some(entry)))
        }
        ty::TyAstNodeContent::SideEffect(_) => Ok(NodeConnection::NextStep(leaf_opt)),
        ty::TyAstNodeContent::Declaration(decl) => Ok(NodeConnection::NextStep(
            connect_declaration(engines, node, decl, graph, leaf_opt)?,
        )),
        ty::TyAstNodeContent::Error(_, _) => Ok(NodeConnection::NextStep(None)),
    }
}

fn connect_declaration<'eng: 'cfg, 'cfg>(
    engines: &'eng Engines,
    node: &ty::TyAstNode,
    decl: &ty::TyDecl,
    graph: &mut ControlFlowGraph<'cfg>,
    leaf_opt: Option<NodeIndex>,
) -> Result<Option<NodeIndex>, Vec<CompileError>> {
    let decl_engine = engines.de();
    match decl {
        ty::TyDecl::TraitDecl(_)
        | ty::TyDecl::AbiDecl(_)
        | ty::TyDecl::StructDecl(_)
        | ty::TyDecl::EnumDecl(_)
        | ty::TyDecl::EnumVariantDecl(_)
        | ty::TyDecl::StorageDecl(_)
        | ty::TyDecl::TypeAliasDecl(_)
        | ty::TyDecl::TraitTypeDecl(_)
        | ty::TyDecl::GenericTypeForFunctionScope(_) => Ok(leaf_opt),
        ty::TyDecl::VariableDecl(_)
        | ty::TyDecl::ConstantDecl(_)
        | ty::TyDecl::ConfigurableDecl(_) => {
            let entry_node = graph.add_node(ControlFlowGraphNode::from_node(node));
            if let Some(leaf) = leaf_opt {
                graph.add_edge(leaf, entry_node, "".into());
            }
            Ok(Some(entry_node))
        }
        ty::TyDecl::ConstGenericDecl(_) => {
            unreachable!("ConstGenericDecl is not reachable from AstNode")
        }
        ty::TyDecl::FunctionDecl(ty::FunctionDecl { decl_id, .. }) => {
            let fn_decl = decl_engine.get_function(decl_id);
            let entry_node = graph.add_node(ControlFlowGraphNode::from_node(node));
            // Do not connect the leaves to the function entry point, since control cannot flow from them into the function.
            connect_typed_fn_decl(engines, &fn_decl, graph, entry_node)?;
            Ok(leaf_opt)
        }
        ty::TyDecl::ImplSelfOrTrait(ty::ImplSelfOrTrait { decl_id, .. }) => {
            let impl_trait = decl_engine.get_impl_self_or_trait(decl_id);
            let ty::TyImplSelfOrTrait {
                trait_name, items, ..
            } = &*impl_trait;
            // Do not connect the leaves to the impl entry point, since control cannot flow from them into the impl.
            connect_impl_trait(engines, trait_name, graph, items)?;
            Ok(leaf_opt)
        }
        ty::TyDecl::ErrorRecovery(..) => Ok(leaf_opt),
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
///
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
        depth_first_insertion_code_block(engines, &fn_decl.body, graph, Some(entry_node))?;
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
    init_leaf_opt: Option<NodeIndex>,
) -> Result<ReturnStatementNodes, Vec<CompileError>> {
    let mut errors = vec![];
    let mut leaf_opt = init_leaf_opt;
    let mut return_nodes = vec![];
    for node in node_content.contents.iter() {
        match connect_node(engines, node, graph, leaf_opt) {
            Ok(this_node) => match this_node {
                NodeConnection::NextStep(node_opt) => leaf_opt = node_opt,
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
