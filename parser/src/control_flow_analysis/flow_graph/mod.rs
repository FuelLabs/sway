//! This is the flow graph, a graph which contains edges that represent possible steps of program
//! execution.

use crate::{
    parse_tree::Ident,
    semantics::ast_node::{TypedEnumVariant, TypedExpressionVariant, TypedTraitDeclaration},
    TreeType,
};
use crate::{
    semantics::{
        ast_node::{
            TypedCodeBlock, TypedDeclaration, TypedEnumDeclaration, TypedExpression,
            TypedFunctionDeclaration, TypedReassignment, TypedVariableDeclaration, TypedWhileLoop,
        },
        TypedAstNode, TypedAstNodeContent, TypedParseTree,
    },
    CompileWarning, Warning,
};
use pest::Span;
use petgraph::algo::has_path_connecting;
use petgraph::{graph::EdgeIndex, prelude::NodeIndex};

mod namespace;
use namespace::ControlFlowNamespace;

pub type EntryPoint = NodeIndex;
pub type ExitPoint = NodeIndex;

pub struct ControlFlowGraph<'sc> {
    graph: Graph<'sc>,
    entry_points: Vec<NodeIndex>,
}

type Graph<'sc> = petgraph::Graph<ControlFlowGraphNode<'sc>, ControlFlowGraphEdge>;

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
    EnumVariant {
        span: Span<'sc>,
        variant_name: String,
    },
}

impl<'sc> std::fmt::Debug for ControlFlowGraphNode<'sc> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let text = match self {
            ControlFlowGraphNode::OrganizationalDominator(s) => s.to_string(),
            ControlFlowGraphNode::ProgramNode(node) => format!("{:?}", node),
            ControlFlowGraphNode::EnumVariant { variant_name, .. } => {
                format!("Enum variant {}", variant_name.to_string())
            }
        };
        f.write_str(&text)
    }
}

impl<'sc> std::convert::From<&TypedAstNode<'sc>> for ControlFlowGraphNode<'sc> {
    fn from(other: &TypedAstNode<'sc>) -> Self {
        ControlFlowGraphNode::ProgramNode(other.clone())
    }
}

impl<'sc> std::convert::From<&TypedEnumVariant<'sc>> for ControlFlowGraphNode<'sc> {
    fn from(other: &TypedEnumVariant<'sc>) -> Self {
        ControlFlowGraphNode::EnumVariant {
            variant_name: other.name.primary_name.to_string(),
            span: other.span.clone(),
        }
    }
}

impl std::convert::From<String> for ControlFlowGraphNode<'_> {
    fn from(other: String) -> Self {
        ControlFlowGraphNode::OrganizationalDominator(other)
    }
}

impl std::convert::From<&str> for ControlFlowGraphNode<'_> {
    fn from(other: &str) -> Self {
        ControlFlowGraphNode::OrganizationalDominator(other.to_string())
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
        let mut namespace = Default::default();
        // do a depth first traversal and cover individual inner ast nodes
        let mut leaves = vec![];
        let exit_node = Some(graph.add_node(("Program exit".to_string()).into()));
        for ast_entrypoint in ast.root_nodes.iter() {
            let (l_leaves, _new_exit_node) = connect_node(
                ast_entrypoint,
                &mut graph,
                &leaves,
                &mut namespace,
                exit_node,
            );

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
        graph.visualize();

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
        let dead_enum_variant_warnings = dead_nodes
            .iter()
            .filter_map(|x| match &self.graph[*x] {
                ControlFlowGraphNode::EnumVariant { span, variant_name } => Some(CompileWarning {
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
                    Some(construct_dead_code_warning_from_node(node))
                }
                _ => None,
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
                all_warnings
                    .iter()
                    .find(
                        |CompileWarning {
                             span: other_span, ..
                         }| {
                            other_span.end() > span.end() && other_span.start() < span.start()
                        },
                    )
                    .is_none()
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
    namespace: &mut ControlFlowNamespace<'sc>,
    exit_node: Option<NodeIndex>,
) -> (Vec<NodeIndex>, Option<NodeIndex>) {
    //    let mut graph = graph.clone();
    let span = node.span.clone();
    match &node.content {
        TypedAstNodeContent::ReturnStatement(_)
        | TypedAstNodeContent::ImplicitReturnExpression(_) => {
            let this_index = graph.add_node(node.into());
            for leaf_ix in leaves {
                graph.add_edge(*leaf_ix, this_index, "".into());
            }
            // connect return to the exit node
            if let Some(exit_node) = exit_node {
                graph.add_edge(this_index, exit_node, "return".into());
                (vec![], None)
            } else {
                (vec![], None)
            }
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
                depth_first_insertion_code_block(body, graph, &leaves, namespace, exit_node);
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
            ..
        }) => {
            let entry = graph.add_node(node.into());
            // insert organizational dominator node
            // connected to all current leaves
            for leaf in leaves {
                graph.add_edge(*leaf, entry, "".into());
            }

            (
                connect_expression(expr_variant, graph, &[entry], namespace, exit_node),
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
                connect_declaration(&decl, graph, decl_node, namespace, span, exit_node),
                exit_node,
            )
        }
    }
}

fn connect_declaration<'sc>(
    decl: &TypedDeclaration<'sc>,
    graph: &mut ControlFlowGraph<'sc>,
    entry_node: NodeIndex,
    namespace: &mut ControlFlowNamespace<'sc>,
    span: Span<'sc>,
    exit_node: Option<NodeIndex>,
) -> Vec<NodeIndex> {
    use TypedDeclaration::*;
    match decl {
        VariableDeclaration(TypedVariableDeclaration { body, .. }) => {
            connect_expression(&body.expression, graph, &[entry_node], namespace, exit_node)
        }
        FunctionDeclaration(fn_decl) => {
            connect_typed_fn_decl(fn_decl, graph, entry_node, namespace, span, exit_node);
            vec![]
        }
        TraitDeclaration(trait_decl) => {
            connect_trait_declaration(&trait_decl, entry_node, namespace);
            vec![]
        }
        StructDeclaration(_) => todo!(),
        EnumDeclaration(enum_decl) => {
            connect_enum_declaration(&enum_decl, graph, entry_node, namespace);
            vec![]
        }
        Reassignment(TypedReassignment { rhs, .. }) => {
            connect_expression(&rhs.expression, graph, &[entry_node], namespace, exit_node)
        }
        SideEffect | ErrorRecovery => {
            unreachable!("These are error cases and should be removed in the type checking stage. ")
        }
    }
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
fn connect_trait_declaration<'sc>(
    decl: &TypedTraitDeclaration<'sc>,
    entry_node: NodeIndex,
    namespace: &mut ControlFlowNamespace<'sc>,
) {
    namespace.add_trait(decl.name.clone(), entry_node);
}

/// For an enum declaration, we want to make a declaration node for every individual enum
/// variant. When a variant is constructed, we can point an edge at that variant. This way,
/// we can see clearly, and thusly warn, when individual variants are not ever constructed.
fn connect_enum_declaration<'sc>(
    enum_decl: &TypedEnumDeclaration<'sc>,
    graph: &mut ControlFlowGraph<'sc>,
    entry_node: NodeIndex,
    namespace: &mut ControlFlowNamespace<'sc>,
) {
    // keep a mapping of each variant
    for variant in &enum_decl.variants {
        let variant_index = graph.add_node(variant.into());

        //        graph.add_edge(entry_node, variant_index, "".into());
        namespace.insert_enum(
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
fn connect_typed_fn_decl<'sc>(
    fn_decl: &TypedFunctionDeclaration<'sc>,
    graph: &mut ControlFlowGraph<'sc>,
    entry_node: NodeIndex,
    namespace: &mut ControlFlowNamespace<'sc>,
    _span: Span<'sc>,
    exit_node: Option<NodeIndex>,
) {
    let fn_exit_node = graph.add_node(format!("\"{}\" fn exit", fn_decl.name.primary_name).into());
    let (_exit_nodes, _exit_node) = depth_first_insertion_code_block(
        &fn_decl.body,
        graph,
        &[entry_node],
        namespace,
        Some(fn_exit_node),
    );
    if let Some(exit_node) = exit_node {
        graph.add_edge(fn_exit_node, exit_node, "".into());
    }

    namespace.insert_function(fn_decl.name.clone(), (entry_node, fn_exit_node));
}

fn depth_first_insertion_code_block<'sc>(
    node_content: &TypedCodeBlock<'sc>,
    graph: &mut ControlFlowGraph<'sc>,
    leaves: &[NodeIndex],
    namespace: &mut ControlFlowNamespace<'sc>,
    exit_node: Option<NodeIndex>,
) -> (Vec<NodeIndex>, Option<NodeIndex>) {
    let mut leaves = leaves.to_vec();
    let mut exit_node = exit_node.clone();
    for node in node_content.contents.iter() {
        let (this_node, l_exit_node) = connect_node(node, graph, &leaves, namespace, exit_node);
        leaves = this_node;
        exit_node = l_exit_node;
    }
    (leaves, exit_node)
}

/// connects any inner parts of an expression to the graph
/// note the main expression node has already been inserted
fn connect_expression<'sc>(
    expr_variant: &TypedExpressionVariant<'sc>,
    graph: &mut ControlFlowGraph<'sc>,
    leaves: &[NodeIndex],
    namespace: &mut ControlFlowNamespace<'sc>,
    exit_node: Option<NodeIndex>,
) -> Vec<NodeIndex> {
    use TypedExpressionVariant::*;
    match expr_variant {
        FunctionApplication { name, .. } => {
            let mut is_external = false;
            // find the function in the namespace
            let (fn_entrypoint, fn_exit_point) = namespace
                .get_function(&name.suffix)
                .cloned()
                .unwrap_or_else(|| {
                    let node_idx =
                        graph.add_node(format!("extern fn {}()", name.suffix.primary_name).into());
                    is_external = true;
                    (node_idx, node_idx)
                });
            for leaf in leaves {
                graph.add_edge(*leaf, fn_entrypoint, "".into());
            }
            // the exit points get connected to an exit node for the application
            // if this is external, then we don't add the body to the graph so there's no point in
            // an exit organizational dominator
            if !is_external {
                if let Some(exit_node) = exit_node {
                    graph.add_edge(fn_exit_point, exit_node, "".into());
                    vec![exit_node]
                } else {
                    vec![fn_exit_point]
                }
            } else {
                vec![fn_entrypoint]
            }
        }
        Literal(_lit) => leaves.to_vec(),
        VariableExpression { .. } => leaves.to_vec(),
        EnumInstantiation {
            enum_name,
            variant_name,
            ..
        } => {
            // connect this particular instantiation to its variants declaration
            connect_enum_instantiation(enum_name, variant_name, graph, namespace, leaves)
        }
        a => todo!("{:?}", a),
    }
}

fn connect_enum_instantiation<'sc>(
    enum_name: &Ident<'sc>,
    variant_name: &Ident<'sc>,
    graph: &mut ControlFlowGraph,
    namespace: &ControlFlowNamespace,
    leaves: &[NodeIndex],
) -> Vec<NodeIndex> {
    let (decl_ix, variant_index) = namespace
        .find_enum_variant_index(enum_name, variant_name)
        .unwrap_or_else(|| {
            let node_idx = graph.add_node(
                format!(
                    "extern enum {}::{}",
                    enum_name.primary_name, variant_name.primary_name
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

    graph.add_edge(decl_ix, variant_index, "".into());
    graph.add_edge(variant_index, enum_instantiation_exit_idx, "".into());

    vec![enum_instantiation_exit_idx]
}

fn construct_dead_code_warning_from_node<'sc>(node: &TypedAstNode<'sc>) -> CompileWarning<'sc> {
    match node {
        // if this is a function, struct, or trait declaration that is never called, then it is dead
        // code.
        TypedAstNode {
            content: TypedAstNodeContent::Declaration(TypedDeclaration::FunctionDeclaration { .. }),
            span,
        } => CompileWarning {
            span: span.clone(),
            warning_content: Warning::DeadDeclaration,
        },
        TypedAstNode {
            content: TypedAstNodeContent::Declaration(TypedDeclaration::StructDeclaration { .. }),
            span,
        } => CompileWarning {
            span: span.clone(),
            warning_content: Warning::DeadDeclaration,
        },
        TypedAstNode {
            content:
                TypedAstNodeContent::Declaration(TypedDeclaration::TraitDeclaration(
                    TypedTraitDeclaration { name, .. },
                )),
            ..
        } => CompileWarning {
            span: name.span.clone(),
            warning_content: Warning::DeadDeclaration,
        },
        TypedAstNode {
            content: TypedAstNodeContent::Declaration(TypedDeclaration::EnumDeclaration(..)),
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
    }
}
