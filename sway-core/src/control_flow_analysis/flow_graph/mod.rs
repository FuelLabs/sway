//! This is the flow graph, a graph which contains edges that represent possible steps of program
//! execution.

use crate::{
    semantic_analysis::{ast_node::TyEnumVariant, ast_node::TyStructField, TyAstNode},
    Ident,
};

use sway_types::span::Span;

use petgraph::{graph::EdgeIndex, prelude::NodeIndex};

mod namespace;
use crate::semantic_analysis::declaration::TyStorageField;
use namespace::ControlFlowNamespace;
pub(crate) use namespace::FunctionNamespaceEntry;

pub type EntryPoint = NodeIndex;
pub type ExitPoint = NodeIndex;

#[derive(Clone, Default)]
/// A graph that can be used to model the control flow of a Sway program.
/// This graph is used as the basis for all of the algorithms in the control flow analysis portion
/// of the compiler.
pub struct ControlFlowGraph {
    pub(crate) graph: Graph,
    pub(crate) entry_points: Vec<NodeIndex>,
    pub(crate) namespace: ControlFlowNamespace,
}

pub type Graph = petgraph::Graph<ControlFlowGraphNode, ControlFlowGraphEdge>;

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
pub enum ControlFlowGraphNode {
    OrganizationalDominator(String),
    #[allow(clippy::large_enum_variant)]
    ProgramNode(TyAstNode),
    EnumVariant {
        variant_name: Ident,
        is_public: bool,
    },
    MethodDeclaration {
        span: Span,
        method_name: Ident,
    },
    StructField {
        struct_field_name: Ident,
        span: Span,
    },
    StorageField {
        field_name: Ident,
    },
}

impl std::fmt::Debug for ControlFlowGraphNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let text = match self {
            ControlFlowGraphNode::OrganizationalDominator(s) => s.to_string(),
            ControlFlowGraphNode::ProgramNode(node) => format!("{:?}", node),
            ControlFlowGraphNode::EnumVariant { variant_name, .. } => {
                format!("Enum variant {}", variant_name)
            }
            ControlFlowGraphNode::MethodDeclaration { method_name, .. } => {
                format!("Method {}", method_name.as_str())
            }
            ControlFlowGraphNode::StructField {
                struct_field_name, ..
            } => {
                format!("Struct field {}", struct_field_name.as_str())
            }
            ControlFlowGraphNode::StorageField { field_name } => {
                format!("Storage field {}", field_name.as_str())
            }
        };
        f.write_str(&text)
    }
}
impl std::convert::From<&TyStorageField> for ControlFlowGraphNode {
    fn from(other: &TyStorageField) -> Self {
        ControlFlowGraphNode::StorageField {
            field_name: other.name.clone(),
        }
    }
}

impl std::convert::From<&TyAstNode> for ControlFlowGraphNode {
    fn from(other: &TyAstNode) -> Self {
        ControlFlowGraphNode::ProgramNode(other.clone())
    }
}

impl std::convert::From<&TyStructField> for ControlFlowGraphNode {
    fn from(other: &TyStructField) -> Self {
        ControlFlowGraphNode::StructField {
            struct_field_name: other.name.clone(),
            span: other.span.clone(),
        }
    }
}
impl std::convert::From<String> for ControlFlowGraphNode {
    fn from(other: String) -> Self {
        ControlFlowGraphNode::OrganizationalDominator(other)
    }
}

impl std::convert::From<&str> for ControlFlowGraphNode {
    fn from(other: &str) -> Self {
        other.to_string().into()
    }
}

impl ControlFlowGraph {
    pub(crate) fn add_edge_from_entry(&mut self, to: NodeIndex, label: ControlFlowGraphEdge) {
        for entry in &self.entry_points {
            self.graph.add_edge(*entry, to, label.clone());
        }
    }
    pub(crate) fn add_node(&mut self, node: ControlFlowGraphNode) -> NodeIndex {
        self.graph.add_node(node)
    }
    pub(crate) fn add_edge(
        &mut self,
        from: NodeIndex,
        to: NodeIndex,
        edge: ControlFlowGraphEdge,
    ) -> EdgeIndex {
        self.graph.add_edge(from, to, edge)
    }

    #[allow(dead_code)]
    /// Prints out graphviz for this graph
    pub(crate) fn visualize(&self) {
        use petgraph::dot::Dot;
        tracing::info!("{:?}", Dot::with_config(&self.graph, &[]));
    }
}

impl ControlFlowGraphNode {
    pub(crate) fn from_enum_variant(
        other: &TyEnumVariant,
        is_public: bool,
    ) -> ControlFlowGraphNode {
        ControlFlowGraphNode::EnumVariant {
            variant_name: other.name.clone(),
            is_public,
        }
    }
}
