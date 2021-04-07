//! This is the flow graph, a graph which contains edges that represent possible steps of program
//! execution.

use crate::{parse_tree::Ident, semantics::ast_node::TypedExpressionVariant, TreeType};
use crate::{
    semantics::{
        ast_node::{
            TypedCodeBlock, TypedDeclaration, TypedExpression, TypedFunctionDeclaration,
            TypedReassignment, TypedVariableDeclaration, TypedWhileLoop,
        },
        TypedAstNode, TypedAstNodeContent, TypedParseTree,
    },
    CompileWarning, Warning,
};
use pest::Span;
use petgraph::algo::has_path_connecting;
use petgraph::{graph::EdgeIndex, prelude::NodeIndex};
use std::collections::HashMap;

pub type EntryPoint = NodeIndex;
pub type ExitPoints = Vec<NodeIndex>;

pub struct ControlFlowGraph<'sc> {
    graph: Graph<'sc>,
    entry_points: Vec<NodeIndex>,
}

type Graph<'sc> = petgraph::Graph<ControlFlowGraphNode<'sc>, ControlFlowGraphEdge>;

pub type ControlFlowFunctionNamespace<'sc> = HashMap<Ident<'sc>, (EntryPoint, ExitPoints)>;

pub struct ControlFlowGraphEdge(String);

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

pub enum ControlFlowGraphNode<'sc> {
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

impl<'sc> ControlFlowGraph<'sc> {
    fn add_node(&mut self, node: ControlFlowGraphNode<'sc>) -> NodeIndex {
        self.graph.add_node(node)
    }
    fn add_edge(
        &mut self,
        from: NodeIndex,
        to: NodeIndex,
        edge: ControlFlowGraphEdge,
    ) -> EdgeIndex {
        self.graph.add_edge(from, to, edge)
    }
    pub(crate) fn from_tree(ast: &TypedParseTree<'sc>, tree_type: TreeType) -> Self {
        let mut graph = ControlFlowGraph {
            graph: Graph::new(),
            entry_points: vec![],
        };
        let mut fn_namespace = Default::default();
        // do a depth first traversal and cover individual inner ast nodes
        let mut leaves = vec![];
        for ast_entrypoint in ast.root_nodes.iter() {
            let l_leaves = connect_node(ast_entrypoint, &mut graph, &leaves, &mut fn_namespace);
            leaves = l_leaves;
        }

        // calculate the entry points based on the tree type
        graph.entry_points =
            match tree_type {
                TreeType::Predicate | TreeType::Script => {
                    // a predicate or script have a main function as the only entry point
                    vec![graph
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
                            }) => name.primary_name == "main",
                            _ => false,
                        })
                        .unwrap()]
                }
                TreeType::Contract | TreeType::Library => {
                    // eventually we want to limit this to pub stuff
                    // TODO issue #17
                    // only pub things are "real" entry points
                    // for now, all functions are entry points

                    vec![graph
                        .graph
                        .node_indices()
                        .find(|i| match graph.graph[*i] {
                            ControlFlowGraphNode::OrganizationalDominator(_) => false,
                            ControlFlowGraphNode::ProgramNode(TypedAstNode {
                                content:
                                    TypedAstNodeContent::Declaration(
                                        TypedDeclaration::FunctionDeclaration(_),
                                    ),
                                ..
                            }) => true,
                            _ => false,
                        })
                        .unwrap()]
                }
            };

        graph
    }

    pub(crate) fn find_dead_code(&self) -> Vec<CompileWarning<'sc>> {
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

        let mut dead_nodes = dead_nodes
            .into_iter()
            .filter_map(|x| match &self.graph[x] {
                ControlFlowGraphNode::OrganizationalDominator(_) => None,
                ControlFlowGraphNode::ProgramNode(node) => Some(node),
            })
            .collect::<Vec<_>>();

        // filter out any overlapping spans -- if a span is contained within another one,
        // remove it.
        dead_nodes = dead_nodes
            .clone()
            .into_iter()
            .filter(|TypedAstNode { span, .. }| {
                // if any other warnings contain a span which completely covers this one, filter
                // out this one.
                dead_nodes
                    .iter()
                    .find(
                        |TypedAstNode {
                             span: other_span, ..
                         }| {
                            other_span.end() > span.end() && other_span.start() < span.start()
                        },
                    )
                    .is_none()
            })
            .collect();
        dead_nodes
            .into_iter()
            .map(|dead_node| CompileWarning {
                span: dead_node.span.clone(),
                warning_content: Warning::DeadCode,
            })
            .collect()
    }

    #[allow(dead_code)]
    /// Prints out graphviz for this graph
    fn visualize(&self) {
        use petgraph::dot::Dot;
        println!("{:?}", Dot::with_config(&self.graph, &[]));
    }
}

fn connect_node<'sc>(
    node: &TypedAstNode<'sc>,
    graph: &mut ControlFlowGraph<'sc>,
    leaves: &[NodeIndex],
    function_namespace: &mut ControlFlowFunctionNamespace<'sc>,
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
            let CodeBlockInsertionResult { leaves: l_leaves } =
                depth_first_insertion_code_block(body, graph, &leaves, function_namespace);
            // insert edges from end of block back to beginning of it
            for leaf in &l_leaves {
                graph.add_edge(*leaf, entry, "loop repeats".into());
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
        }) => {
            let entry = graph.add_node(node.into());
            // insert organizational dominator node
            // connected to all current leaves
            for leaf in leaves {
                graph.add_edge(*leaf, entry, "".into());
            }

            connect_expression(expr_variant, graph, &[entry], function_namespace)
        }
        TypedAstNodeContent::SideEffect => leaves.to_vec(),
        TypedAstNodeContent::Declaration(decl) => {
            // all leaves connect to this node, then this node is the singular leaf
            let decl_node = graph.add_node(node.into());
            for leaf in leaves {
                graph.add_edge(*leaf, decl_node, "".into());
            }
            connect_declaration(&decl, graph, decl_node, function_namespace, span)
        }
    }
}

fn connect_declaration<'sc>(
    decl: &TypedDeclaration<'sc>,
    graph: &mut ControlFlowGraph<'sc>,
    entry_node: NodeIndex,
    function_namespace: &mut ControlFlowFunctionNamespace<'sc>,
    span: Span<'sc>,
) -> Vec<NodeIndex> {
    use TypedDeclaration::*;
    match decl {
        VariableDeclaration(TypedVariableDeclaration { body, .. }) => {
            connect_expression(&body.expression, graph, &[entry_node], function_namespace)
        }
        FunctionDeclaration(fn_decl) => {
            connect_typed_fn_decl(fn_decl, graph, entry_node, function_namespace, span);
            vec![]
        }
        TraitDeclaration(_) => todo!(),
        StructDeclaration(_) => todo!(),
        EnumDeclaration(_) => todo!(),
        Reassignment(TypedReassignment { rhs, .. }) => {
            connect_expression(&rhs.expression, graph, &[entry_node], function_namespace)
        }
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
    entry_node: NodeIndex,
    function_namespace: &mut ControlFlowFunctionNamespace<'sc>,
    _span: Span<'sc>,
) {
    let CodeBlockInsertionResult {
        leaves: exit_nodes, ..
    } = depth_first_insertion_code_block(&fn_decl.body, graph, &[entry_node], function_namespace);

    function_namespace.insert(fn_decl.name.clone(), (entry_node, exit_nodes));
}

fn depth_first_insertion_code_block<'sc>(
    node_content: &TypedCodeBlock<'sc>,
    graph: &mut ControlFlowGraph<'sc>,
    leaves: &[NodeIndex],
    fn_namespace: &mut ControlFlowFunctionNamespace<'sc>,
) -> CodeBlockInsertionResult {
    let mut leaves = leaves.to_vec();
    for node in node_content.contents.iter() {
        let this_node = connect_node(node, graph, &leaves, fn_namespace);
        leaves = this_node;
    }
    CodeBlockInsertionResult { leaves }
}

/// connects any inner parts of an expression to the graph
/// note the main expression node has already been inserted
fn connect_expression<'sc>(
    expr_variant: &TypedExpressionVariant<'sc>,
    graph: &mut ControlFlowGraph<'sc>,
    leaves: &[NodeIndex],
    fn_namespace: &mut ControlFlowFunctionNamespace<'sc>,
) -> Vec<NodeIndex> {
    use TypedExpressionVariant::*;
    match expr_variant {
        FunctionApplication { name, .. } => {
            let mut is_external = false;
            // find the function in the namespace
            let (fn_entrypoint, fn_exit_points) =
                fn_namespace.get(&name.suffix).cloned().unwrap_or_else(|| {
                    let node_idx =
                        graph.add_node(format!("extern fn {}()", name.suffix.primary_name).into());
                    is_external = true;
                    (node_idx, vec![node_idx])
                });
            for leaf in leaves {
                graph.add_edge(*leaf, fn_entrypoint, "".into());
            }
            // the exit points get connected to an exit node for the application
            // if this is external, then we don't add the body to the graph so there's no point in
            // an exit organizational dominator
            if !is_external {
                let exit =
                    graph.add_node(format!("\"{}\" fn exit", name.suffix.primary_name).into());
                for exit_point in fn_exit_points {
                    graph.add_edge(exit_point, exit, "".into());
                }

                vec![exit]
            } else {
                vec![fn_entrypoint]
            }
        }
        Literal(_lit) => leaves.to_vec(),
        VariableExpression { .. } => leaves.to_vec(),
        a => todo!("{:?}", a),
    }
}

struct CodeBlockInsertionResult {
    leaves: Vec<NodeIndex>,
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
