//! This is the flow graph, a graph which contains edges that represent possible steps of program
//! execution.

use super::*;
use super::{ControlFlowGraph, EntryPoint, ExitPoint, Graph};
use crate::semantics::{
    ast_node::{
        TypedCodeBlock, TypedDeclaration, TypedEnumDeclaration, TypedExpression,
        TypedFunctionDeclaration, TypedReassignment, TypedVariableDeclaration, TypedWhileLoop,
    },
    TypedAstNode, TypedAstNodeContent,
};
use crate::{error::*, semantics::TypedParseTree};
use crate::{
    semantics::ast_node::{TypedExpressionVariant, TypedTraitDeclaration},
    Ident,
};
use pest::Span;
use petgraph::prelude::NodeIndex;

impl<'sc> ControlFlowGraph<'sc> {
    pub(crate) fn construct_return_path_graph(ast: &TypedParseTree<'sc>) -> Self {
        let mut graph = ControlFlowGraph {
            graph: Graph::new(),
            entry_points: vec![],
            namespace: Default::default(),
        };
        // do a depth first traversal and cover individual inner ast nodes
        let mut leaves = vec![];
        let exit_node = Some(graph.add_node(("Program exit".to_string()).into()));
        for ast_entrypoint in ast.root_nodes.iter() {
            let (l_leaves, _new_exit_node) =
                connect_node(ast_entrypoint, &mut graph, &leaves, exit_node);

            leaves = l_leaves;
        }

        graph
    }
    /// This function  looks through the control flow graph and ensures that all paths that are
    /// required to return a value do, indeed, return a value of the correct type.
    /// It does this by checking every function declaration in both the methods namespace
    /// and the functions namespace and validating that all paths leading to the function exit node
    /// return the same type. Additionally, if a function has a return type, all paths must indeed
    /// lead to the function exit node.
    pub(crate) fn analyze_return_paths(&self) -> Vec<CompileError<'sc>> {
        let mut errors = vec![];
        for (_name, (entry_point, exit_point)) in &self.namespace.function_namespace {
            // For every node connected to the entry point
            errors.append(&mut self.ensure_all_paths_reach_exit(*entry_point, *exit_point));
        }
        errors
    }
    fn ensure_all_paths_reach_exit(
        &self,
        entry_point: EntryPoint,
        exit_point: ExitPoint,
    ) -> Vec<CompileError<'sc>> {
        let mut rovers = vec![entry_point];
        let errors = vec![];
        let mut max_iterations = 50;
        while rovers.len() >= 1 && rovers[0] != exit_point && max_iterations > 0 {
            max_iterations -= 1;
            dbg!(&rovers);
            rovers = rovers
                .into_iter()
                .filter(|idx| *idx != exit_point)
                .collect();
            let mut next_rovers = vec![];
            for rover in rovers {
                let mut neighbors = self
                    .graph
                    .neighbors_directed(rover, petgraph::Direction::Outgoing)
                    .collect::<Vec<_>>();
                if neighbors.is_empty() {
                    //j                    errors.push(todo!("Path does not return error"));
                }
                next_rovers.append(&mut neighbors);
            }
            rovers = next_rovers;
        }

        errors
    }
}
fn connect_node<'sc>(
    node: &TypedAstNode<'sc>,
    graph: &mut ControlFlowGraph<'sc>,
    leaves: &[NodeIndex],
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
                depth_first_insertion_code_block(body, graph, &leaves, exit_node);
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
                connect_expression(expr_variant, graph, &[entry], exit_node),
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
                connect_declaration(&decl, graph, decl_node, span, exit_node),
                exit_node,
            )
        }
    }
}

fn connect_declaration<'sc>(
    decl: &TypedDeclaration<'sc>,
    graph: &mut ControlFlowGraph<'sc>,
    entry_node: NodeIndex,
    span: Span<'sc>,
    exit_node: Option<NodeIndex>,
) -> Vec<NodeIndex> {
    use TypedDeclaration::*;
    match decl {
        VariableDeclaration(TypedVariableDeclaration { body, .. }) => {
            connect_expression(&body.expression, graph, &[entry_node], exit_node)
        }
        FunctionDeclaration(fn_decl) => {
            connect_typed_fn_decl(fn_decl, graph, entry_node, span, exit_node);
            vec![]
        }
        TraitDeclaration(trait_decl) => {
            connect_trait_declaration(&trait_decl, graph, entry_node);
            vec![]
        }
        StructDeclaration(_) => todo!("track each struct field's usage"),
        EnumDeclaration(enum_decl) => {
            connect_enum_declaration(&enum_decl, graph, entry_node);
            vec![]
        }
        Reassignment(TypedReassignment { rhs, .. }) => {
            connect_expression(&rhs.expression, graph, &[entry_node], exit_node)
        }
        ImplTrait {
            trait_name,
            methods,
            ..
        } => {
            connect_impl_trait(trait_name, graph, methods, entry_node);
            vec![]
        }
        SideEffect | ErrorRecovery => {
            unreachable!("These are error cases and should be removed in the type checking stage. ")
        }
    }
}

/// Implementations of traits are top-level things that are not conditional, so
/// we insert an edge from the function's starting point to the declaration to show
/// that the declaration was indeed at some point implemented.
/// Additionally, we insert the trait's methods into the method namespace in order to
/// track which exact methods are dead code.
fn connect_impl_trait<'sc>(
    trait_name: &Ident<'sc>,
    graph: &mut ControlFlowGraph<'sc>,
    methods: &[TypedFunctionDeclaration<'sc>],
    entry_node: NodeIndex,
) {
    let graph_c = graph.clone();
    let trait_decl_node = graph_c.namespace.find_trait(trait_name);
    match trait_decl_node {
        None => {
            let edge_ix = graph.add_node("External trait".into());
            graph.add_edge(entry_node, edge_ix, "".into());
        }
        Some(trait_decl_node) => {
            graph.add_edge_from_entry(entry_node, "".into());
            graph.add_edge(entry_node, *trait_decl_node, "".into());
        }
    }
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
            &fn_decl,
            graph,
            fn_decl_entry_node,
            fn_decl.span.clone(),
            None,
        );
        methods_and_indexes.push((fn_decl.name.clone(), fn_decl_entry_node));
    }
    // Now, insert the methods into the trait method namespace.
    graph
        .namespace
        .insert_trait_methods(trait_name.clone(), methods_and_indexes);
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
    graph: &mut ControlFlowGraph<'sc>,
    entry_node: NodeIndex,
) {
    graph.namespace.add_trait(decl.name.clone(), entry_node);
}

/// For an enum declaration, we want to make a declaration node for every individual enum
/// variant. When a variant is constructed, we can point an edge at that variant. This way,
/// we can see clearly, and thusly warn, when individual variants are not ever constructed.
fn connect_enum_declaration<'sc>(
    enum_decl: &TypedEnumDeclaration<'sc>,
    graph: &mut ControlFlowGraph<'sc>,
    entry_node: NodeIndex,
) {
    // keep a mapping of each variant
    for variant in &enum_decl.variants {
        let variant_index = graph.add_node(variant.into());

        //        graph.add_edge(entry_node, variant_index, "".into());
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
fn connect_typed_fn_decl<'sc>(
    fn_decl: &TypedFunctionDeclaration<'sc>,
    graph: &mut ControlFlowGraph<'sc>,
    entry_node: NodeIndex,
    _span: Span<'sc>,
    exit_node: Option<NodeIndex>,
) {
    let fn_exit_node = graph.add_node(format!("\"{}\" fn exit", fn_decl.name.primary_name).into());
    let (_exit_nodes, _exit_node) =
        depth_first_insertion_code_block(&fn_decl.body, graph, &[entry_node], Some(fn_exit_node));
    if let Some(exit_node) = exit_node {
        graph.add_edge(fn_exit_node, exit_node, "".into());
    }

    graph
        .namespace
        .insert_function(fn_decl.name.clone(), (entry_node, fn_exit_node));
}

fn depth_first_insertion_code_block<'sc>(
    node_content: &TypedCodeBlock<'sc>,
    graph: &mut ControlFlowGraph<'sc>,
    leaves: &[NodeIndex],
    exit_node: Option<NodeIndex>,
) -> (Vec<NodeIndex>, Option<NodeIndex>) {
    let mut leaves = leaves.to_vec();
    let mut exit_node = exit_node.clone();
    for node in node_content.contents.iter() {
        let (this_node, l_exit_node) = connect_node(node, graph, &leaves, exit_node);
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
    exit_node: Option<NodeIndex>,
) -> Vec<NodeIndex> {
    use TypedExpressionVariant::*;
    match expr_variant {
        FunctionApplication { name, .. } => {
            let mut is_external = false;
            // find the function in the namespace
            let (fn_entrypoint, fn_exit_point) = graph
                .namespace
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
            connect_enum_instantiation(enum_name, variant_name, graph, leaves)
        }
        a => todo!("{:?}", a),
    }
}

fn connect_enum_instantiation<'sc>(
    enum_name: &Ident<'sc>,
    variant_name: &Ident<'sc>,
    graph: &mut ControlFlowGraph,
    leaves: &[NodeIndex],
) -> Vec<NodeIndex> {
    let (decl_ix, variant_index) = graph
        .namespace
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
