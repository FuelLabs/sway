//! This module handles the process of iterating through the typed AST and doing an analysis.
//! At the moment we compute an dependency graph between typed nodes.

use std::collections::HashMap;
use std::fs;

use sway_error::handler::{ErrorEmitted, Handler};

use crate::decl_engine::{DeclId, DeclIdIndexType, DeclRef};
use crate::engine_threading::DebugWithEngines;
use crate::language::ty::{self, TyImplItem, TyTraitItem};
use crate::Engines;

pub type TyNodeDepGraphNodeId = petgraph::graph::NodeIndex;

#[derive(Clone)]
pub struct TyNodeDepGraphEdge(pub String);

#[allow(clippy::large_enum_variant)]
#[derive(Clone, Debug)]
pub enum TyNodeDepGraphNode {
    ImplTrait { node: ty::ImplTrait },
    ImplTraitItem { node: ty::TyTraitItem },
}

// Represents an ordered graph between declaration id indexes.
pub type TyNodeDepGraph = petgraph::graph::DiGraph<TyNodeDepGraphNode, TyNodeDepGraphEdge>;

// A simple context that is used to pass context necessary for typed AST analysis.
pub struct TypeCheckAnalysisContext<'cx> {
    pub(crate) engines: &'cx Engines,
    pub(crate) dep_graph: TyNodeDepGraph,
    pub(crate) nodes: HashMap<DeclIdIndexType, TyNodeDepGraphNodeId>,
    pub(crate) items_node_stack: Vec<TyNodeDepGraphNodeId>,
    pub(crate) node_stack: Vec<TyNodeDepGraphNodeId>,
}

impl TypeCheckAnalysisContext<'_> {
    pub fn add_node(&mut self, node: TyNodeDepGraphNode) -> TyNodeDepGraphNodeId {
        self.dep_graph.add_node(node)
    }

    pub fn add_edge_from_current(&mut self, a: TyNodeDepGraphNodeId, edge: TyNodeDepGraphEdge) {
        self.dep_graph
            .add_edge(*self.node_stack.last().unwrap(), a, edge);
    }

    pub(crate) fn push_impl_trait(&mut self, impl_trait: &ty::ImplTrait) -> TyNodeDepGraphNodeId {
        let node = if self.nodes.contains_key(&impl_trait.decl_id.inner()) {
            *self.nodes.get(&impl_trait.decl_id.inner()).unwrap()
        } else {
            self.add_node(TyNodeDepGraphNode::ImplTrait {
                node: impl_trait.clone(),
            })
        };

        let decl_engine = self.engines.de();
        let impl_trait = decl_engine.get_impl_trait(&impl_trait.decl_id);

        for item in impl_trait.items.iter() {
            let decl_id = match item {
                TyTraitItem::Fn(node) => node.id().inner(),
                TyTraitItem::Constant(node) => node.id().inner(),
                TyTraitItem::Type(node) => node.id().inner(),
            };

            let item_node = if self.nodes.contains_key(&decl_id) {
                *self.nodes.get(&decl_id).unwrap()
            } else {
                self.add_node(TyNodeDepGraphNode::ImplTraitItem { node: item.clone() })
            };

            // Connect the item node to the impl trait node.
            self.dep_graph
                .add_edge(node, item_node, TyNodeDepGraphEdge(String::from("")));

            self.items_node_stack.push(item_node);
        }

        node
    }

    #[allow(dead_code)]
    pub(crate) fn get_node_from_impl_trait_item(
        &self,
        item: &TyImplItem,
    ) -> Option<TyNodeDepGraphNodeId> {
        for index in self.items_node_stack.iter().rev() {
            let node = self
                .dep_graph
                .node_weight(*index)
                .expect("expecting valid node id");
            if let TyNodeDepGraphNode::ImplTraitItem { node } = node {
                let matches = match (item, node) {
                    (TyTraitItem::Fn(item_fn_ref), TyTraitItem::Fn(fn_ref)) => {
                        fn_ref.name() == item_fn_ref.name()
                    }
                    _ => unreachable!(),
                };
                if matches {
                    return Some(*index);
                }
            }
        }

        None
    }

    pub(crate) fn get_node_from_impl_trait_fn_ref_app(
        &self,
        fn_ref: &DeclRef<DeclId<ty::TyFunctionDecl>>,
    ) -> Option<TyNodeDepGraphNodeId> {
        for index in self.items_node_stack.iter().rev() {
            let node = self
                .dep_graph
                .node_weight(*index)
                .expect("expecting valid node id");
            if let TyNodeDepGraphNode::ImplTraitItem {
                node: TyTraitItem::Fn(item_fn_ref),
            } = node
            {
                if fn_ref.name() == item_fn_ref.name() {
                    return Some(*index);
                }
            }
        }

        None
    }

    /// Prints out GraphViz DOT format for the dependency graph.
    #[allow(dead_code)]
    pub(crate) fn visualize(
        &self,
        engines: &Engines,
        print_graph: Option<String>,
        print_graph_url_format: Option<String>,
    ) {
        if let Some(graph_path) = print_graph {
            use petgraph::dot::{Config, Dot};
            let string_graph = self.dep_graph.filter_map(
                |_idx, node| Some(format!("{:?}", engines.help_out(node))),
                |_idx, edge| Some(edge.0.clone()),
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
                        // if self.entry_points.contains(&nr.0) {
                        //     shape = "shape=doubleoctagon";
                        // }
                        let url = "".to_string();
                        if let Some(_url_format) = print_graph_url_format.clone() {
                            // if let Some(span) = node.span() {
                            //     if let Some(source_id) = span.source_id() {
                            //         let path = engines.se().get_path(source_id);
                            //         let path = path.to_string_lossy();
                            //         let (line, col) = span.start_pos().line_col();
                            //         let url_format = url_format
                            //             .replace("{path}", path.to_string().as_str())
                            //             .replace("{line}", line.to_string().as_str())
                            //             .replace("{col}", col.to_string().as_str());
                            //         url = format!("URL = {url_format:?}");
                            //     }
                            // }
                        }
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
                        "There was an issue while outputing type check analysis graph to path {graph_path:?}\n{error}"
                    );
                }
            }
        }
    }

    #[allow(dead_code)]
    pub(crate) fn sort_nodes(&self) {
        let _res = petgraph::algo::toposort(&self.dep_graph, None)
            .expect("found a cycle in the dependency graph");
    }
}

impl DebugWithEngines for TyNodeDepGraphNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>, _engines: &Engines) -> std::fmt::Result {
        let text = match self {
            TyNodeDepGraphNode::ImplTraitItem { node } => {
                let str = match node {
                    ty::TyTraitItem::Fn(node) => node.name().as_str(),
                    ty::TyTraitItem::Constant(node) => node.name().as_str(),
                    ty::TyTraitItem::Type(node) => node.name().as_str(),
                };
                format!("{:?}", str)
            }
            TyNodeDepGraphNode::ImplTrait { node } => {
                format!("{:?}", node.name.as_str())
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
