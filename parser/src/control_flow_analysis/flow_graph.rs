//! This is the flow graph, a graph which contains edges that represent possible steps of program
//! execution.

use crate::parse_tree::Ident;
use crate::semantics::{
    ast_node::{
        TypedCodeBlock, TypedDeclaration, TypedExpression, TypedFunctionDeclaration, TypedWhileLoop,
    },
    TypedAstNode, TypedAstNodeContent, TypedParseTree,
};
use pest::Span;
use petgraph::prelude::NodeIndex;
use petgraph::Graph;
use std::collections::HashMap;

type ControlFlowGraph<'sc> = Graph<TypedAstNode<'sc>, ()>;
type FunctionNamespace<'sc> = HashMap<Ident<'sc>, NodeIndex>;

fn construct_graph<'sc>(ast: TypedParseTree<'sc>) -> ControlFlowGraph<'sc> {
    let mut graph = ControlFlowGraph::new();
    let mut fn_namespace = Default::default();
    // do a depth first traversal and cover individual inner ast nodes
    let mut leaves = vec![];
    for ast_entrypoint in ast.root_nodes.iter() {
        let l_leaves = connect_node(ast_entrypoint, &mut graph, &leaves, &mut fn_namespace);
        leaves = l_leaves;
    }
    todo!()
}

fn connect_node<'sc>(
    node: &TypedAstNode<'sc>,
    graph: &mut ControlFlowGraph<'sc>,
    leaves: &[NodeIndex],
    function_namespace: &mut FunctionNamespace<'sc>,
) -> Vec<NodeIndex> {
    //    let mut graph = graph.clone();
    let span = node.span.clone();
    match &node.content {
        TypedAstNodeContent::ReturnStatement(_) => {
            let this_index = graph.add_node(node.clone());
            for leaf_ix in leaves {
                graph.add_edge(*leaf_ix, this_index, ());
            }
            vec![this_index]
        }
        TypedAstNodeContent::WhileLoop(TypedWhileLoop { body, .. }) => {
            // a while loop can loop back to the beginning,
            // or it can terminate.
            // so we connect the _end_ of the while loop _both_ to its beginning and the next node.
            // the loop could also be entirely skipped

            let CodeBlockInsertionResult {
                leaves: l_leaves,
                first_node_in_block,
            } = depth_first_insertion_code_block(body, graph, leaves, function_namespace);
            // insert edges from end of block back to beginning of it
            for leaf in &l_leaves {
                for first_node in &first_node_in_block {
                    graph.add_edge(*leaf, *first_node, ());
                }
            }

            // current leaves are the leaves before the while loop and the leaves after the while
            // loop, since it is possible for an entire while loop to be skipped.
            [leaves, &l_leaves].concat().to_vec()
        }
        TypedAstNodeContent::Declaration(TypedDeclaration::FunctionDeclaration(fn_decl)) => {
            connect_typed_fn_decl(fn_decl, graph, function_namespace, span);
            leaves.to_vec()
        }
        TypedAstNodeContent::Expression(TypedExpression {
            expression: _expr_variant,
            ..
        }) => {
            todo!("match on the expression (in another function) and handle function calls using the namespace, etc.")
        }
        TypedAstNodeContent::SideEffect => leaves.to_vec(),
        TypedAstNodeContent::Declaration(_) => leaves.to_vec(),
        _ => todo!(),
    }
}

/// When connecting a function declaration, we are inserting a new root node into the graph that
/// has no entry points, since it is just a declaration.
/// When something eventually calls it, it gets connected to the declaration.
fn connect_typed_fn_decl<'sc>(
    fn_decl: &TypedFunctionDeclaration<'sc>,
    graph: &mut ControlFlowGraph<'sc>,
    function_namespace: &mut FunctionNamespace<'sc>,
    span: Span<'sc>,
) {
    let entry_node = graph.add_node(TypedAstNode {
        span,
        content: TypedAstNodeContent::Declaration(TypedDeclaration::FunctionDeclaration(
            fn_decl.clone(),
        )),
    });

    let _code_block_res =
        depth_first_insertion_code_block(&fn_decl.body, graph, &[entry_node], function_namespace);

    function_namespace.insert(fn_decl.name.clone(), entry_node);
}

fn depth_first_insertion_code_block<'sc>(
    node_content: &TypedCodeBlock<'sc>,
    graph: &mut ControlFlowGraph<'sc>,
    leaves: &[NodeIndex],
    fn_namespace: &mut FunctionNamespace<'sc>,
) -> CodeBlockInsertionResult {
    let mut leaves = leaves.to_vec();
    let mut is_first = true;
    let mut first_node_in_block = vec![];
    for node in node_content.contents.iter() {
        if is_first {
            first_node_in_block = connect_node(node, graph, &leaves, fn_namespace);
            for next_leaf in first_node_in_block.iter() {
                for prev_leaf in leaves.iter() {
                    graph.add_edge(*prev_leaf, *next_leaf, ());
                }
            }
            leaves = first_node_in_block.clone();
            is_first = false;
        } else {
            let l_leaves = connect_node(node, graph, &leaves, fn_namespace);
            leaves = l_leaves;
        }
    }
    CodeBlockInsertionResult {
        leaves,
        first_node_in_block,
    }
}

struct CodeBlockInsertionResult {
    leaves: Vec<NodeIndex>,
    /// Keep a handle to the first node in the block
    /// so we can connect loops back to their first node
    first_node_in_block: Vec<NodeIndex>,
}
/*
fn connect_node<'sc>(
    node_content: TypedAstNodeContent<'sc>,
    graph: &mut ControlFlowGraph<'sc>,
    leaves: &[NodeIndex],
) -> Vec<NodeIndex> {
    todo!()
}
struct FlowGraph<'a> {
    root_node: FlowGraphNode<'a>
}


struct FlowGraphNode<'a> {
    syntax_tree_node: &'a TypedAstNode,
    next_node: Box<FlowGraphNode>
}
*/
