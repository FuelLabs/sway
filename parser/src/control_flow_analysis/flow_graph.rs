//! This is the flow graph, a graph which contains edges that represent possible steps of program
//! execution.

use crate::semantics::{
    ast_node::{
        TypedCodeBlock, TypedDeclaration, TypedExpression, TypedFunctionDeclaration, TypedWhileLoop,
    },
    TypedAstNode, TypedAstNodeContent, TypedParseTree,
};
use crate::{parse_tree::Ident, semantics::ast_node::TypedExpressionVariant};
use pest::Span;
use petgraph::prelude::NodeIndex;
use petgraph::Graph;
use std::collections::HashMap;

type EntryPoint = NodeIndex;
type ExitPoints = Vec<NodeIndex>;
type ControlFlowGraph<'sc> = Graph<ControlFlowGraphNode<'sc>, ControlFlowGraphEdge>;
type FunctionNamespace<'sc> = HashMap<Ident<'sc>, (EntryPoint, ExitPoints)>;

pub(crate) struct ControlFlowGraphEdge(String);
impl std::fmt::Debug for ControlFlowGraphEdge {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::convert::From<&str> for ControlFlowGraphEdge {
    fn from(o: &str) -> Self {
        ControlFlowGraphEdge(o.to_string())
    }
}

pub(crate) enum ControlFlowGraphNode<'sc> {
    OrganizationalDominator(String),
    ProgramNode(TypedAstNode<'sc>),
}

impl<'sc> std::fmt::Debug for ControlFlowGraphNode<'sc> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let text = match self {
            ControlFlowGraphNode::OrganizationalDominator(s) => s.to_string(),
            ControlFlowGraphNode::ProgramNode(node) => format!("{:?}", node),
        };
        f.write_str(&text)
    }
}

impl<'sc> std::convert::From<&TypedAstNode<'sc>> for ControlFlowGraphNode<'sc> {
    fn from(other: &TypedAstNode<'sc>) -> Self {
        ControlFlowGraphNode::ProgramNode(other.clone())
    }
}

impl std::convert::From<String> for ControlFlowGraphNode<'_> {
    fn from(other: String) -> Self {
        ControlFlowGraphNode::OrganizationalDominator(other)
    }
}

pub(crate) fn construct_graph<'sc>(ast: &TypedParseTree<'sc>) -> ControlFlowGraph<'sc> {
    let mut graph = ControlFlowGraph::new();
    let mut fn_namespace = Default::default();
    // do a depth first traversal and cover individual inner ast nodes
    let mut leaves = vec![];
    for ast_entrypoint in ast.root_nodes.iter() {
        let l_leaves = connect_node(ast_entrypoint, &mut graph, &leaves, &mut fn_namespace);
        leaves = l_leaves;
    }

    visualize(&graph);
    todo!()
}

fn visualize(graph: &ControlFlowGraph) {
    use petgraph::dot::Dot;
    println!("{:?}", Dot::with_config(&graph, &[]));
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
        TypedAstNodeContent::ReturnStatement(_)
        | TypedAstNodeContent::ImplicitReturnExpression(_) => {
            let this_index = graph.add_node(node.into());
            for leaf_ix in leaves {
                graph.add_edge(*leaf_ix, this_index, "".into());
            }
            vec![this_index]
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
            let CodeBlockInsertionResult {
                leaves: l_leaves,
                first_node_in_block,
            } = depth_first_insertion_code_block(body, graph, &leaves, function_namespace);
            // insert edges from end of block back to beginning of it
            for leaf in &l_leaves {
                for first_node in &first_node_in_block {
                    graph.add_edge(*leaf, *first_node, "loop repeats".into());
                }
            }

            leaves = l_leaves;
            for leaf in leaves {
                graph.add_edge(leaf, while_loop_exit, "".into());
            }
            vec![while_loop_exit]
        }
        TypedAstNodeContent::Expression(TypedExpression {
            expression: expr_variant,
            ..
        }) => connect_expression(&node, expr_variant, graph, &leaves, function_namespace),
        TypedAstNodeContent::SideEffect => leaves.to_vec(),
        TypedAstNodeContent::Declaration(decl) => {
            // all leaves connect to this node, then this node is the singular leaf
            let decl_node = graph.add_node(node.into());
            connect_declaration(&decl, graph, &[decl_node], function_namespace, span)
        }
        a => todo!("Unimplemented control flow: {:?}", a),
    }
}

fn connect_declaration<'sc>(
    decl: &TypedDeclaration<'sc>,
    graph: &mut ControlFlowGraph<'sc>,
    leaves: &[NodeIndex],
    function_namespace: &mut FunctionNamespace<'sc>,
    span: Span<'sc>,
) -> Vec<NodeIndex> {
    use TypedDeclaration::*;
    match decl {
        VariableDeclaration(_) => todo!(),
        FunctionDeclaration(fn_decl) => {
            connect_typed_fn_decl(fn_decl, graph, function_namespace, span);
            vec![]
        }
        TraitDeclaration(_) => todo!(),
        StructDeclaration(_) => todo!(),
        EnumDeclaration(_) => todo!(),
        Reassignment(_) => todo!(),
        SideEffect | ErrorRecovery => {
            unreachable!("These are error cases and should be removed in the type checking stage. ")
        }
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
    let entry_node = graph.add_node(
        (&TypedAstNode {
            span,
            content: TypedAstNodeContent::Declaration(TypedDeclaration::FunctionDeclaration(
                fn_decl.clone(),
            )),
        })
            .into(),
    );

    let CodeBlockInsertionResult {
        leaves: exit_nodes, ..
    } = depth_first_insertion_code_block(&fn_decl.body, graph, &[entry_node], function_namespace);

    function_namespace.insert(fn_decl.name.clone(), (entry_node, exit_nodes));
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
        let this_node = connect_node(node, graph, &leaves, fn_namespace);
        if is_first {
            first_node_in_block = this_node.clone();
            is_first = false;
        }
        leaves = this_node;
    }
    CodeBlockInsertionResult {
        leaves,
        first_node_in_block,
    }
}

fn connect_expression<'sc>(
    node: &TypedAstNode<'sc>,
    expr_variant: &TypedExpressionVariant<'sc>,
    graph: &mut ControlFlowGraph<'sc>,
    leaves: &[NodeIndex],
    fn_namespace: &mut FunctionNamespace<'sc>,
) -> Vec<NodeIndex> {
    use TypedExpressionVariant::*;
    match expr_variant {
        FunctionApplication { name, .. } => {
            // insert organizational dominator node
            // connected to all current leaves
            let entry = graph.add_node(node.into());
            for leaf in leaves {
                graph.add_edge(*leaf, entry, "".into());
            }

            // find the function in the namespace
            let (fn_entrypoint, fn_exit_points) = fn_namespace.get(&name.suffix).expect(
                "calling nonexistent functions should have been caught in the type checking stage",
            );
            // the current leaves all get connected to the entry of the function
            graph.add_edge(entry, *fn_entrypoint, "".into());
            // the exit points get connected to an exit node for the application
            let exit = graph.add_node(format!("\"{}\" fn exit", name.suffix.primary_name).into());
            for exit_point in fn_exit_points {
                graph.add_edge(*exit_point, exit, "".into());
            }

            vec![exit]
        }
        _ => todo!(),
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
