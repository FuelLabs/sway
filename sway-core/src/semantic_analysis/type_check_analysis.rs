//! This module handles the process of iterating through the typed AST and doing an analysis.
//! At the moment, we compute a dependency graph between typed nodes.

use std::collections::{HashMap, HashSet};
use std::fmt::Display;
use std::fs;

use petgraph::stable_graph::NodeIndex;
use petgraph::Graph;
use sway_error::error::CompileError;
use sway_error::handler::{ErrorEmitted, Handler};
use sway_types::Named;

use crate::decl_engine::{AssociatedItemDeclId, DeclId, DeclUniqueId};
use crate::engine_threading::DebugWithEngines;
use crate::language::ty::{self, TyFunctionDecl, TyTraitItem};
use crate::Engines;

use graph_cycles::Cycles;

pub type TyNodeDepGraphNodeId = petgraph::graph::NodeIndex;

#[derive(Clone, Debug)]
pub enum TyNodeDepGraphEdgeInfo {
    FnApp,
}

#[derive(Clone, Debug)]
pub struct TyNodeDepGraphEdge(pub TyNodeDepGraphEdgeInfo);

impl Display for TyNodeDepGraphEdge {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.0 {
            TyNodeDepGraphEdgeInfo::FnApp => write!(f, "fn app"),
        }
    }
}

#[derive(Clone, Debug)]
pub enum TyNodeDepGraphNode {
    ImplTrait { node: ty::ImplSelfOrTrait },
    ImplTraitItem { node: ty::TyTraitItem },
    Fn { node: DeclId<TyFunctionDecl> },
}

// Represents an ordered graph between declaration id indexes.
pub type TyNodeDepGraph = petgraph::graph::DiGraph<TyNodeDepGraphNode, TyNodeDepGraphEdge>;

// A simple context that is used to pass context necessary for typed AST analysis.
pub struct TypeCheckAnalysisContext<'cx> {
    pub(crate) engines: &'cx Engines,
    pub(crate) dep_graph: TyNodeDepGraph,
    pub(crate) nodes: HashMap<DeclUniqueId, TyNodeDepGraphNodeId>,
    pub(crate) items_node_stack: Vec<TyNodeDepGraphNodeId>,
    pub(crate) node_stack: Vec<TyNodeDepGraphNodeId>,
}

impl TypeCheckAnalysisContext<'_> {
    pub fn add_node(&mut self, node: TyNodeDepGraphNode) -> TyNodeDepGraphNodeId {
        self.dep_graph.add_node(node)
    }

    pub fn add_edge_from_current(&mut self, to: TyNodeDepGraphNodeId, edge: TyNodeDepGraphEdge) {
        let from = *self.node_stack.last().unwrap();
        if !self.dep_graph.contains_edge(from, to) {
            self.dep_graph.add_edge(from, to, edge);
        }
    }

    #[allow(clippy::map_entry)]
    pub fn get_or_create_node_for_impl_item(&mut self, item: &TyTraitItem) -> TyNodeDepGraphNodeId {
        let id = match item {
            TyTraitItem::Fn(decl_ref) => decl_ref.id().unique_id(),
            TyTraitItem::Constant(decl_ref) => decl_ref.id().unique_id(),
            TyTraitItem::Type(decl_ref) => decl_ref.id().unique_id(),
        };
        if self.nodes.contains_key(&id) {
            *self.nodes.get(&id).unwrap()
        } else {
            let item_node = self.add_node(TyNodeDepGraphNode::ImplTraitItem { node: item.clone() });

            self.nodes.insert(id, item_node);
            item_node
        }
    }

    /// This functions either gets an existing node in the graph, or creates a new
    /// node corresponding to the passed function declaration node.
    /// The function will try to find a non-monomorphized declaration node id so that
    /// future accesses always normalize to the same node id.
    #[allow(clippy::map_entry)]
    pub fn get_or_create_node_for_fn_decl(
        &mut self,
        fn_decl_id: &DeclId<TyFunctionDecl>,
    ) -> TyNodeDepGraphNodeId {
        let parents = self
            .engines
            .de()
            .find_all_parents(self.engines, fn_decl_id)
            .into_iter()
            .filter_map(|f| match f {
                AssociatedItemDeclId::TraitFn(_) => None,
                AssociatedItemDeclId::Function(fn_id) => Some(fn_id),
                AssociatedItemDeclId::Constant(_) => None,
                AssociatedItemDeclId::Type(_) => None,
            })
            .collect::<Vec<_>>();
        let id = if !parents.is_empty() {
            parents.first().unwrap().unique_id()
        } else {
            fn_decl_id.unique_id()
        };
        if self.nodes.contains_key(&id) {
            *self.nodes.get(&id).unwrap()
        } else {
            let item_node = self.add_node(TyNodeDepGraphNode::Fn { node: *fn_decl_id });

            self.nodes.insert(id, item_node);
            item_node
        }
    }

    /// This function will process an impl trait declaration, pushing graph nodes
    /// corresponding to each item in the trait impl.
    #[allow(clippy::map_entry)]
    pub(crate) fn push_nodes_for_impl_trait(
        &mut self,
        impl_trait: &ty::ImplSelfOrTrait,
    ) -> TyNodeDepGraphNodeId {
        if self.nodes.contains_key(&impl_trait.decl_id.unique_id()) {
            *self.nodes.get(&impl_trait.decl_id.unique_id()).unwrap()
        } else {
            let node = self.add_node(TyNodeDepGraphNode::ImplTrait {
                node: impl_trait.clone(),
            });
            self.nodes.insert(impl_trait.decl_id.unique_id(), node);

            let decl_engine = self.engines.de();
            let impl_trait = decl_engine.get_impl_self_or_trait(&impl_trait.decl_id);

            for item in impl_trait.items.iter() {
                let item_node = self.get_or_create_node_for_impl_item(item);

                // Connect the item node to the impl trait node.
                self.dep_graph.add_edge(
                    node,
                    item_node,
                    TyNodeDepGraphEdge(TyNodeDepGraphEdgeInfo::FnApp),
                );

                self.items_node_stack.push(item_node);
            }

            node
        }
    }

    /// This function will return an option to the node that represents the
    /// function being referenced by a function application.
    /// It will look through all the parent nodes in the engine to deal with
    /// monomorphized function references.
    pub(crate) fn get_node_for_fn_decl(
        &mut self,
        fn_decl_id: &DeclId<TyFunctionDecl>,
    ) -> Option<TyNodeDepGraphNodeId> {
        let parents = self
            .engines
            .de()
            .find_all_parents(self.engines, fn_decl_id)
            .into_iter()
            .filter_map(|f| match f {
                AssociatedItemDeclId::TraitFn(_) => None,
                AssociatedItemDeclId::Function(fn_id) => Some(fn_id),
                AssociatedItemDeclId::Constant(_) => None,
                AssociatedItemDeclId::Type(_) => None,
            })
            .collect::<Vec<_>>();

        let mut possible_nodes = vec![*fn_decl_id];
        possible_nodes.append(&mut parents.clone());

        for possible_node in possible_nodes.iter().rev() {
            if let Some(found) = self.nodes.get(&possible_node.unique_id()) {
                return Some(*found);
            }
        }

        for index in self.items_node_stack.iter().rev() {
            let node = self
                .dep_graph
                .node_weight(*index)
                .expect("expecting valid node id");

            let fn_decl_id = match node {
                TyNodeDepGraphNode::ImplTrait { node: _ } => unreachable!(),
                TyNodeDepGraphNode::ImplTraitItem {
                    node: TyTraitItem::Fn(item_fn_ref),
                } => item_fn_ref.id(),
                TyNodeDepGraphNode::Fn { node: fn_decl_id } => fn_decl_id,
                _ => continue,
            };

            for possible_node in possible_nodes.iter() {
                if possible_node.inner() == fn_decl_id.inner() {
                    return Some(*index);
                }
            }
        }

        // If no node has been found yet, create it.
        let base_id = if !parents.is_empty() {
            parents.first().unwrap()
        } else {
            fn_decl_id
        };
        let node = self.get_or_create_node_for_fn_decl(base_id);
        Some(node)
    }

    /// Prints out GraphViz DOT format for the dependency graph.
    #[allow(dead_code)]
    pub(crate) fn visualize(&self, engines: &Engines, print_graph: Option<String>) {
        if let Some(graph_path) = print_graph {
            use petgraph::dot::{Config, Dot};
            let string_graph = self.dep_graph.filter_map(
                |_idx, node| Some(format!("{:?}", engines.help_out(node))),
                |_idx, edge| Some(format!("{edge}")),
            );

            let output = format!(
                "{:?}",
                Dot::with_attr_getters(
                    &string_graph,
                    &[Config::NodeNoLabel, Config::EdgeNoLabel],
                    &|_, er| format!("label = {:?}", er.weight()),
                    &|_, nr| {
                        let _node = &self.dep_graph[nr.0];
                        let shape = "";
                        let url = "".to_string();
                        format!("{shape} label = {:?} {url}", nr.1)
                    },
                )
            );

            if graph_path.is_empty() {
                tracing::info!("{output}");
            } else {
                let result = fs::write(graph_path.clone(), output);
                if let Some(error) = result.err() {
                    tracing::error!(
                        "There was an issue while outputting type check analysis graph to path {graph_path:?}\n{error}"
                    );
                }
            }
        }
    }

    /// Performs recursive analysis by running the Johnson's algorithm to find all cycles
    /// in the previously constructed dependency graph.
    pub(crate) fn check_recursive_calls(&self, handler: &Handler) -> Result<(), ErrorEmitted> {
        handler.scope(|handler| {
            let cycles = self.dep_graph.cycles();
            if cycles.is_empty() {
                return Ok(());
            }
            for mut sub_cycles in cycles {
                // Manipulate the cycles order to get the same ordering as the source code's lexical order.
                sub_cycles.rotate_left(1);
                if sub_cycles.len() == 1 {
                    let node = self.dep_graph.node_weight(sub_cycles[0]).unwrap();
                    let fn_decl_id = self.get_fn_decl_id_from_node(node);

                    let fn_decl = self.engines.de().get_function(&fn_decl_id);
                    handler.emit_err(CompileError::RecursiveCall {
                        fn_name: fn_decl.name.clone(),
                        span: fn_decl.span.clone(),
                    });
                } else {
                    let node = self.dep_graph.node_weight(sub_cycles[0]).unwrap();

                    let mut call_chain = vec![];
                    for i in sub_cycles.into_iter().skip(1) {
                        let node = self.dep_graph.node_weight(i).unwrap();
                        let fn_decl_id = self.get_fn_decl_id_from_node(node);
                        let fn_decl = self.engines.de().get_function(&fn_decl_id);
                        call_chain.push(fn_decl.name.to_string());
                    }

                    let fn_decl_id = self.get_fn_decl_id_from_node(node);
                    let fn_decl = self.engines.de().get_function(&fn_decl_id);
                    handler.emit_err(CompileError::RecursiveCallChain {
                        fn_name: fn_decl.name.clone(),
                        call_chain: call_chain.join(" -> "),
                        span: fn_decl.span.clone(),
                    });
                }
            }
            Ok(())
        })
    }

    pub(crate) fn get_normalized_fn_node_id(
        &self,
        fn_decl_id: &DeclId<TyFunctionDecl>,
    ) -> DeclId<TyFunctionDecl> {
        let parents = self
            .engines
            .de()
            .find_all_parents(self.engines, fn_decl_id)
            .into_iter()
            .filter_map(|f| match f {
                AssociatedItemDeclId::TraitFn(_) => None,
                AssociatedItemDeclId::Function(fn_id) => Some(fn_id),
                AssociatedItemDeclId::Constant(_) => None,
                AssociatedItemDeclId::Type(_) => None,
            })
            .collect::<Vec<_>>();

        if !parents.is_empty() {
            self.get_normalized_fn_node_id(parents.first().unwrap())
        } else {
            *fn_decl_id
        }
    }

    pub(crate) fn get_fn_decl_id_from_node(
        &self,
        node: &TyNodeDepGraphNode,
    ) -> DeclId<TyFunctionDecl> {
        match node {
            TyNodeDepGraphNode::ImplTrait { .. } => unreachable!(),
            TyNodeDepGraphNode::ImplTraitItem { node } => match node {
                TyTraitItem::Fn(node) => *node.id(),
                TyTraitItem::Constant(_) => unreachable!(),
                TyTraitItem::Type(_) => unreachable!(),
            },
            TyNodeDepGraphNode::Fn { node } => *node,
        }
    }

    pub(crate) fn get_sub_graph(
        &self,
        node_index: NodeIndex,
    ) -> Graph<&TyNodeDepGraphNode, &TyNodeDepGraphEdge> {
        let neighbors: Vec<_> = self
            .dep_graph
            .neighbors_directed(node_index, petgraph::Direction::Outgoing)
            .collect();
        let neighbors_set: HashSet<&NodeIndex> = HashSet::from_iter(neighbors.iter());
        self.dep_graph.filter_map(
            |node_index, node| {
                if neighbors_set.contains(&node_index) {
                    Some(node)
                } else {
                    None
                }
            },
            |_edge_index, edge| Some(edge),
        )
    }
}

impl DebugWithEngines for TyNodeDepGraphNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>, engines: &Engines) -> std::fmt::Result {
        let text = match self {
            TyNodeDepGraphNode::ImplTraitItem { node } => {
                let str = match node {
                    ty::TyTraitItem::Fn(node) => node.name().as_str(),
                    ty::TyTraitItem::Constant(node) => node.name().as_str(),
                    ty::TyTraitItem::Type(node) => node.name().as_str(),
                };
                format!("{str:?}")
            }
            TyNodeDepGraphNode::ImplTrait { node } => {
                let decl = engines.de().get_impl_self_or_trait(&node.decl_id);
                format!("{:?}", decl.name().as_str())
            }
            TyNodeDepGraphNode::Fn { node } => {
                let fn_decl = engines.de().get_function(node);
                format!("{:?}", fn_decl.name.as_str())
            }
        };
        f.write_str(&text)
    }
}

impl<'cx> TypeCheckAnalysisContext<'cx> {
    pub fn new(engines: &'cx Engines) -> Self {
        Self {
            engines,
            dep_graph: Default::default(),
            nodes: Default::default(),
            items_node_stack: Default::default(),
            node_stack: Default::default(),
        }
    }
}

pub(crate) trait TypeCheckAnalysis {
    fn type_check_analyze(
        &self,
        handler: &Handler,
        ctx: &mut TypeCheckAnalysisContext,
    ) -> Result<(), ErrorEmitted>;
}
