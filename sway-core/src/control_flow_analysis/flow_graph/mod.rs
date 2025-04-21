//! This is the flow graph, a graph which contains edges that represent possible steps of program
//! execution.

use std::{collections::HashMap, fs};

use crate::{
    decl_engine::*,
    engine_threading::DebugWithEngines,
    language::ty::{self, GetDeclIdent},
    transform, Engines, Ident,
};

use sway_types::{span::Span, BaseIdent, IdentUnique, LineCol, Spanned};

use petgraph::{graph::EdgeIndex, prelude::NodeIndex};

mod namespace;
use namespace::ControlFlowNamespace;
pub(crate) use namespace::FunctionNamespaceEntry;
pub(crate) use namespace::TraitNamespaceEntry;
pub(crate) use namespace::VariableNamespaceEntry;

pub type EntryPoint = NodeIndex;
pub type ExitPoint = NodeIndex;

#[derive(Clone)]
/// A graph that can be used to model the control flow of a Sway program.
/// This graph is used as the basis for all of the algorithms in the control flow analysis portion
/// of the compiler.
pub struct ControlFlowGraph<'cfg> {
    pub(crate) graph: Graph<'cfg>,
    pub(crate) entry_points: Vec<NodeIndex>,
    pub(crate) pending_entry_points_edges: Vec<(NodeIndex, ControlFlowGraphEdge)>,
    pub(crate) namespace: ControlFlowNamespace,
    pub(crate) decls: HashMap<IdentUnique, NodeIndex>,
    pub(crate) engines: &'cfg Engines,
}

pub type Graph<'cfg> = petgraph::Graph<ControlFlowGraphNode<'cfg>, ControlFlowGraphEdge>;

impl<'cfg> ControlFlowGraph<'cfg> {
    pub fn new(engines: &'cfg Engines) -> Self {
        Self {
            graph: Default::default(),
            entry_points: Default::default(),
            pending_entry_points_edges: Default::default(),
            namespace: Default::default(),
            decls: Default::default(),
            engines,
        }
    }
}

#[derive(Clone)]
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

#[allow(clippy::large_enum_variant)]
#[derive(Clone)]
pub enum ControlFlowGraphNode<'cfg> {
    OrganizationalDominator(String),
    #[allow(clippy::large_enum_variant)]
    ProgramNode {
        node: ty::TyAstNode,
        parent_node: Option<NodeIndex>,
    },
    EnumVariant {
        enum_decl_id: DeclId<ty::TyEnumDecl>,
        variant_name: Ident,
        is_public: bool,
    },
    MethodDeclaration {
        span: Span,
        method_name: Ident,
        method_decl_ref: DeclRefFunction,
        engines: &'cfg Engines,
    },
    StructField {
        struct_decl_id: DeclId<ty::TyStructDecl>,
        struct_field_name: Ident,
        attributes: transform::Attributes,
    },
    StorageField {
        field_name: Ident,
    },
    FunctionParameter {
        param_name: Ident,
        is_self: bool,
    },
}

impl GetDeclIdent for ControlFlowGraphNode<'_> {
    fn get_decl_ident(&self, engines: &Engines) -> Option<Ident> {
        match self {
            ControlFlowGraphNode::OrganizationalDominator(_) => None,
            ControlFlowGraphNode::ProgramNode { node, .. } => node.get_decl_ident(engines),
            ControlFlowGraphNode::EnumVariant { variant_name, .. } => Some(variant_name.clone()),
            ControlFlowGraphNode::MethodDeclaration { method_name, .. } => {
                Some(method_name.clone())
            }
            ControlFlowGraphNode::StructField {
                struct_field_name, ..
            } => Some(struct_field_name.clone()),
            ControlFlowGraphNode::StorageField { field_name, .. } => Some(field_name.clone()),
            ControlFlowGraphNode::FunctionParameter { param_name, .. } => Some(param_name.clone()),
        }
    }
}

impl std::convert::From<&ty::TyStorageField> for ControlFlowGraphNode<'_> {
    fn from(other: &ty::TyStorageField) -> Self {
        ControlFlowGraphNode::StorageField {
            field_name: other.name.clone(),
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
        other.to_string().into()
    }
}

impl DebugWithEngines for ControlFlowGraphNode<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>, engines: &Engines) -> std::fmt::Result {
        let text = match self {
            ControlFlowGraphNode::OrganizationalDominator(s) => s.to_string(),
            ControlFlowGraphNode::ProgramNode { node, .. } => {
                format!("{:?}", engines.help_out(node))
            }
            ControlFlowGraphNode::EnumVariant { variant_name, .. } => {
                format!("Enum variant {variant_name}")
            }
            ControlFlowGraphNode::MethodDeclaration {
                method_name,
                method_decl_ref,
                engines,
                ..
            } => {
                let decl_engines = engines.de();
                let method = decl_engines.get_function(method_decl_ref);
                if let Some(implementing_type) = &method.implementing_type {
                    format!(
                        "Method {}.{}",
                        implementing_type.friendly_name(engines),
                        method_name.as_str()
                    )
                } else {
                    format!("Method {}", method_name.as_str())
                }
            }
            ControlFlowGraphNode::StructField {
                struct_field_name, ..
            } => {
                format!("Struct field {}", struct_field_name.as_str())
            }
            ControlFlowGraphNode::StorageField { field_name } => {
                format!("Storage field {}", field_name.as_str())
            }
            ControlFlowGraphNode::FunctionParameter { param_name, .. } => {
                format!("Function param {}", param_name.as_str())
            }
        };
        f.write_str(&text)
    }
}

impl<'cfg> ControlFlowGraph<'cfg> {
    pub(crate) fn add_edge_from_entry(&mut self, to: NodeIndex, label: ControlFlowGraphEdge) {
        self.pending_entry_points_edges.push((to, label));
    }
    pub(crate) fn add_node<'eng: 'cfg>(&mut self, node: ControlFlowGraphNode<'cfg>) -> NodeIndex {
        let ident_opt = node.get_decl_ident(self.engines);
        let node_index = self.graph.add_node(node);
        if let Some(ident) = ident_opt {
            self.decls.insert(ident.into(), node_index);
        }
        node_index
    }
    pub(crate) fn add_edge(
        &mut self,
        from: NodeIndex,
        to: NodeIndex,
        edge: ControlFlowGraphEdge,
    ) -> EdgeIndex {
        self.graph.add_edge(from, to, edge)
    }

    pub(crate) fn connect_pending_entry_edges(&mut self) {
        for entry in &self.entry_points {
            for (to, label) in &self.pending_entry_points_edges {
                self.graph.add_edge(*entry, *to, label.clone());
            }
        }
        self.pending_entry_points_edges.clear();
    }

    pub(crate) fn get_node_from_decl(&self, cfg_node: &ControlFlowGraphNode) -> Option<NodeIndex> {
        if let Some(ident) = cfg_node.get_decl_ident(self.engines) {
            if !ident.span().is_dummy() {
                self.decls.get(&ident.into()).cloned()
            } else {
                None
            }
        } else {
            None
        }
    }

    /// Prints out GraphViz DOT format for this graph.
    pub(crate) fn visualize(
        &self,
        engines: &Engines,
        print_graph: Option<String>,
        print_graph_url_format: Option<String>,
    ) {
        if let Some(graph_path) = print_graph {
            use petgraph::dot::{Config, Dot};
            let string_graph = self.graph.filter_map(
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
                        let node = &self.graph[nr.0];
                        let mut shape = "";
                        if self.entry_points.contains(&nr.0) {
                            shape = "shape=doubleoctagon";
                        }
                        let mut url = "".to_string();
                        if let Some(url_format) = print_graph_url_format.clone() {
                            if let Some(span) = node.span() {
                                if let Some(source_id) = span.source_id() {
                                    let path = engines.se().get_path(source_id);
                                    let path = path.to_string_lossy();
                                    let LineCol { line, col } = span.start_line_col_one_index();
                                    let url_format = url_format
                                        .replace("{path}", path.to_string().as_str())
                                        .replace("{line}", line.to_string().as_str())
                                        .replace("{col}", col.to_string().as_str());
                                    url = format!("URL = {url_format:?}");
                                }
                            }
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
                        "There was an issue while outputting DCA graph to path {graph_path:?}\n{error}"
                    );
                }
            }
        }
    }
}

impl<'cfg> ControlFlowGraphNode<'cfg> {
    pub(crate) fn from_enum_variant(
        enum_decl_id: DeclId<ty::TyEnumDecl>,
        other_name: BaseIdent,
        is_public: bool,
    ) -> ControlFlowGraphNode<'cfg> {
        ControlFlowGraphNode::EnumVariant {
            enum_decl_id,
            variant_name: other_name,
            is_public,
        }
    }

    pub(crate) fn from_node_with_parent(
        node: &ty::TyAstNode,
        parent_node: Option<NodeIndex>,
    ) -> ControlFlowGraphNode<'cfg> {
        ControlFlowGraphNode::ProgramNode {
            node: node.clone(),
            parent_node,
        }
    }

    pub(crate) fn from_node(node: &ty::TyAstNode) -> ControlFlowGraphNode<'cfg> {
        ControlFlowGraphNode::ProgramNode {
            node: node.clone(),
            parent_node: None,
        }
    }

    fn span(&self) -> Option<Span> {
        match self {
            ControlFlowGraphNode::OrganizationalDominator(_) => None,
            ControlFlowGraphNode::ProgramNode { node, .. } => Some(node.span.clone()),
            ControlFlowGraphNode::EnumVariant { variant_name, .. } => Some(variant_name.span()),
            ControlFlowGraphNode::MethodDeclaration { span, .. } => Some(span.clone()),
            ControlFlowGraphNode::StructField {
                struct_field_name, ..
            } => Some(struct_field_name.span()),
            ControlFlowGraphNode::StorageField { field_name } => Some(field_name.span()),
            ControlFlowGraphNode::FunctionParameter { param_name, .. } => Some(param_name.span()),
        }
    }
}
