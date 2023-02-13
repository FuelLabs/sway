use std::collections::HashMap;

use petgraph::{stable_graph::StableGraph, Directed};
use sway_types::Span;

use crate::language::{ty::TyProgram, DepName};

type GraphIx = u32;
type NodeIx = petgraph::graph::NodeIndex<GraphIx>;

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub(crate) struct Node {
    pub(crate) dep_decls: Vec<NodeIx>,
    pub(crate) dep_name: DepName
}

pub(crate) type DepGraph = StableGraph<Node, (), Directed, GraphIx>;

#[derive(Debug)]
pub(crate) struct SubmodulePlan {
    pub(crate) graph: DepGraph,
}

impl SubmodulePlan {
    /// Constructs a new `SubmodulePlan` from given `TyProgram`.
    pub(crate) fn from_ty_program(ty_program: &TyProgram) -> Self {
        let root = &ty_program.root;
        let mut graph = DepGraph::new();
        let mut node_indices = HashMap::new();

        // Add non-root nodes to the graph.
        for (dep_name, submodule) in root.submodules_recursive() {
            let dep_decls = &submodule.module.submodules;
            let dep_decls: Vec<NodeIx> = dep_decls.iter().map(|(dep_decl, _)| node_indices[dep_decl]).collect();
            let node = Node::new(dep_name.clone(), dep_decls);
            node_indices.entry(dep_name).or_insert_with(|| {
                graph.add_node(node.clone())
            });
        }

        // Create the root node.
        let root_node_name = DepName::new(Span::dummy());
        let root_node_dep_decls = root.submodules.iter().filter_map(|(dep_name, _)| graph.node_indices().find(|node| graph[*node].dep_name == *dep_name)).collect();
        let root_node = Node::new(root_node_name.clone(), root_node_dep_decls);

        // Add root node.
        let root_ix = graph.add_node(root_node);
        node_indices.insert(&root_node_name, root_ix);

        // Add edges to the graph.
        for node_ix in node_indices.values() {
            let node_neighbors = graph[*node_ix].dep_decls.clone();
            for neighbor in node_neighbors {
                graph.add_edge(*node_ix, neighbor, ());
            }
        }

        Self {
            graph,
        }
    }

    /// Checks validity of this `SubmodulePlan`. If there are more nodes that has more than 1
    /// incoming edge to it, the plan is marked as invalid. This prevents duplicate module
    /// declarations both directly and transitively. 
    pub(crate) fn check_validity(self) -> bool {
        let graph = self.graph;
        for node in graph.node_indices() {
            let incoming_edges_count = graph.edges_directed(node, petgraph::Direction::Incoming).count();
            if incoming_edges_count > 1 {
                return false;
            }
        }
        true
    }

}

impl Node {
    /// Construct a new `Node` from given `TyModule`.
    fn new(dep_name: DepName, dep_decls: Vec<NodeIx>) -> Self {
        Self {
            dep_decls,
            dep_name,
        }
    }
}
